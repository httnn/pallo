extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Error, Ident, ImplItem, ImplItemFn, ItemImpl, PathArguments, Token, Type,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
};
use syn::{LitStr, parse_macro_input};

use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[proc_macro]
pub fn property_id(input: TokenStream) -> TokenStream {
    let input_literal = parse_macro_input!(input as LitStr);

    let id = COUNTER.fetch_add(1, Ordering::SeqCst);

    let expanded = quote! {
        PropertyId::new(#id, #input_literal)
    };

    TokenStream::from(expanded)
}

enum ChildSpec {
    Plain(Ident),
    Opt(Ident),
    Iter(Ident),
}

impl ChildSpec {
    fn ident(&self) -> &Ident {
        match self {
            ChildSpec::Plain(i) | ChildSpec::Opt(i) | ChildSpec::Iter(i) => i,
        }
    }
}

struct ChildrenArgs {
    children: Vec<ChildSpec>,
}

impl Parse for ChildrenArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let list: Punctuated<ChildSpec, Token![,]> = input.parse_terminated(ChildSpec::parse, Token![,])?;
        Ok(Self { children: list.into_iter().collect() })
    }
}

impl Parse for ChildSpec {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;

        if input.peek(Token![?]) {
            let _q: Token![?] = input.parse()?;
            return Ok(ChildSpec::Opt(ident));
        }
        if input.peek(Token![*]) {
            let _s: Token![*] = input.parse()?;
            return Ok(ChildSpec::Iter(ident));
        }
        Ok(ChildSpec::Plain(ident))
    }
}

fn extract_ui_type(item_impl: &ItemImpl) -> syn::Result<Type> {
    let trait_path = item_impl
        .trait_
        .as_ref()
        .ok_or_else(|| Error::new(item_impl.span(), "#[children] must be on a trait impl"))?
        .1
        .segments
        .last()
        .ok_or_else(|| Error::new(item_impl.span(), "unexpected empty trait path"))?;

    if trait_path.ident != "Component" {
        return Err(Error::new(trait_path.ident.span(), "#[children] expects an impl of Component<UI>"));
    }

    match &trait_path.arguments {
        PathArguments::AngleBracketed(ab) => {
            let first =
                ab.args.first().ok_or_else(|| Error::new(ab.span(), "Component<UI> must have a UI type argument"))?;
            match first {
                syn::GenericArgument::Type(ty) => Ok(ty.clone()),
                other => Err(Error::new(other.span(), "expected a type argument")),
            }
        }
        _ => Err(Error::new(trait_path.arguments.span(), "expected Component<UI>")),
    }
}

fn has_method(items: &[ImplItem], name: &str) -> bool {
    items.iter().any(|it| matches!(it, ImplItem::Fn(f) if f.sig.ident == name))
}

fn parse2_or_compile_error<T: syn::parse::Parse>(ts: proc_macro2::TokenStream) -> Result<T, TokenStream> {
    syn::parse2::<T>(ts).map_err(|e| e.to_compile_error().into())
}

fn compile_error(msg: &str) -> TokenStream {
    let lit = proc_macro2::Literal::string(msg);
    quote::quote!( compile_error!(#lit); ).into()
}

#[proc_macro_attribute]
pub fn children(attr: TokenStream, item: TokenStream) -> TokenStream {
    let result = std::panic::catch_unwind(|| children_impl(attr, item));
    match result {
        Ok(ts) => ts,
        Err(panic) => {
            let msg = if let Some(s) = panic.downcast_ref::<&'static str>() {
                format!("children macro panicked: {s}")
            } else if let Some(s) = panic.downcast_ref::<String>() {
                format!("children macro panicked: {s}")
            } else {
                "children macro panicked (non-string payload)".to_string()
            };
            compile_error(&msg)
        }
    }
}

fn children_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ChildrenArgs);
    let mut item_impl = parse_macro_input!(item as ItemImpl);

    let ui_ty = match extract_ui_type(&item_impl) {
        Ok(t) => t,
        Err(e) => return e.to_compile_error().into(),
    };

    let needs_draw = !has_method(&item_impl.items, "draw_children");
    let needs_event = !has_method(&item_impl.items, "event_children");

    let draw_stmts = args.children.iter().map(|c| {
        let field = c.ident();
        match c {
            ChildSpec::Plain(_) => quote! { self.#field.draw(cx, canvas); },
            ChildSpec::Opt(_) => quote! {
                if let Some(child) = self.#field.as_ref() {
                    child.draw(cx, canvas);
                }
            },
            ChildSpec::Iter(_) => quote! {
                for child in (&self.#field).into_iter() {
                    child.draw(cx, canvas);
                }
            },
        }
    });

    let event_stmts = args.children.iter().map(|c| {
        let field = c.ident();
        match c {
            ChildSpec::Plain(_) => quote! { self.#field.event(cx, event); },
            ChildSpec::Opt(_) => quote! {
                if let Some(child) = self.#field.as_mut() {
                    child.event(cx, event);
                }
            },
            ChildSpec::Iter(_) => quote! {
                for child in (&mut self.#field).into_iter() {
                    child.event(cx, event);
                }
            },
        }
    });

    let mut injected: Vec<ImplItem> = Vec::new();

    if needs_draw {
        let f: ImplItemFn = match parse2_or_compile_error(quote! {
            fn draw_children(&self, cx: &mut Cx<#ui_ty>, canvas: &mut Canvas) {
                #(#draw_stmts)*
            }
        }) {
            Ok(f) => f,
            Err(ts) => return ts,
        };
        injected.push(ImplItem::Fn(f));
    }

    if needs_event {
        let f: ImplItemFn = match parse2_or_compile_error(quote! {
            fn event_children(&mut self, cx: &mut Cx<#ui_ty>, event: &mut Event<#ui_ty>) {
                #(#event_stmts)*
            }
        }) {
            Ok(f) => f,
            Err(ts) => return ts,
        };
        injected.push(ImplItem::Fn(f));
    }

    if !injected.is_empty() {
        let mut new_items = injected;
        new_items.extend(item_impl.items.into_iter());
        item_impl.items = new_items;
    }

    TokenStream::from(quote! { #item_impl })
}
