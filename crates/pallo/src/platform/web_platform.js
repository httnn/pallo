export class JsFile {
  constructor(name, data) {
    this.name = name;
    this.data = data;
  }
  get_name() { return this.name; }
  get_data() { return this.data; }
}

export class JsView {
  constructor() {
    this.canvas = document.createElement('canvas');
    this.surface = CanvasKit.MakeWebGLCanvasSurface(this.canvas);
  }

  resize(width, height) {
    this.canvas.width = width;
    this.canvas.height = height;
    this.canvas.tabIndex = 0;

    this.canvas.style.maxWidth = `${width}px`;
    this.canvas.style.maxHeight = `${height}px`;
    this.canvas.style.aspectRatio = `${width} / ${height}`;
    
    this.canvas.width *= window.devicePixelRatio;
    this.canvas.height *= window.devicePixelRatio;

    this.surface = CanvasKit.MakeWebGLCanvasSurface(this.canvas);
  }
}

export function save_file(filename, bytes, mime_type) {
  const a = document.createElement('a');
  a.setAttribute('download', filename);
  const blob = new Blob([bytes], { type: mime_type });
  a.href = URL.createObjectURL(blob);
  a.click();
}

export function get_file_input() {
  let input = document.querySelector('#pallo-file-picker');
  if (!input) {
    input = document.createElement('input');
    input.type = 'file';
    input.id = 'pallo-file-picker';
    input.style = 'display: none !important;';
    document.body.appendChild(input);
  }
  return input;
}

export function trigger_file_input(callback) {
  const input = get_file_input();

  input.addEventListener('change', async e => {
    const files = [];
    for (const file of e.target.files) {
      const name = file.webkitRelativePath || file.name;
      const data = await new Promise(ok => {
        const reader = new FileReader();
        reader.onload = () => {
          ok(new Uint8Array(reader.result));
        };
        reader.readAsArrayBuffer(file);
      });
      files.push(new JsFile(name, data));
    }
    callback(files);
  }, { once: true });

  input.click();
}

export function create_canvas_internal(ui, width, height) {
  const supportsTouch = 'ontouchstart' in window;

  const canvasNode = ui.get_view().canvas;
  ui.get_view().resize(width, height);
  ui.on_resize(width, height, window.devicePixelRatio);

  function getPointerPosition(e) {
    const rect = canvasNode.getBoundingClientRect();
    const clientX = e.clientX || e.touches?.[0].clientX || 0;
    const clientY = e.clientY || e.touches?.[0].clientY || 0;
    const relativeX = (clientX - rect.left) / rect.width;
    const relativeY = (clientY - rect.top) / rect.height;
    const x = relativeX * +canvasNode.width / window.devicePixelRatio;
    const y = relativeY * +canvasNode.height / window.devicePixelRatio;
    return { x, y };
  }

  if (supportsTouch) {
    canvasNode.addEventListener('touchstart', e => {
      const {x, y} = getPointerPosition(e);
      ui.mouse_down(x, y, false);
    });
  
    canvasNode.addEventListener('touchmove', e => {
      if (e.touches.length === 1) {
        e.preventDefault();
        const {x, y} = getPointerPosition(e);
        ui.mouse_move(x, y);  
      }
    }, { passive: false });
  
    canvasNode.addEventListener('touchend', e => {
      ui.mouse_up();
    });
  } else {
    canvasNode.addEventListener('mousedown', e => {
      if (e.button === 0) {
        const {x, y} = getPointerPosition(e);
        ui.mouse_down(x, y, false);
      }
    });

    canvasNode.addEventListener('contextmenu', e => {
      e.preventDefault();
      const {x, y} = getPointerPosition(e);
      ui.mouse_down(x, y, true);
    });

    window.addEventListener('mouseup', e => {
      ui.mouse_up();
    });

    window.addEventListener('mousemove', e => {
      const {x, y} = getPointerPosition(e);
      ui.mouse_move(x, y);
    });
  }

  canvasNode.addEventListener('wheel', e => {
    e.preventDefault();
    e.stopPropagation();
    ui.mouse_wheel(e.deltaX, -e.deltaY);
  });

  canvasNode.addEventListener('focus', e => {
    ui.focus(true);
  });

  canvasNode.addEventListener('blur', e => {
    ui.focus(false);
  });

  window.addEventListener('keydown', e => {
    if (document.activeElement === canvasNode) {
      ui.modifiers_changed(e.metaKey, e.shiftKey, e.altKey);
      if (ui.key_down(e.key)) {
        e.stopPropagation();
        e.preventDefault();
      }
    }
  });

  window.addEventListener('keyup', e => {
    if (document.activeElement === canvasNode) {
      ui.modifiers_changed(e.metaKey, e.shiftKey, e.altKey);
      if (ui.key_up(e.key)) {
        e.stopPropagation();
        e.preventDefault();
      }
    }
  });

  document.addEventListener('dragover', e => {
    const {x, y} = getPointerPosition(e);
    ui.mouse_move(x, y);

    if (e.target === canvasNode) {
      e.preventDefault();
      if (e.dataTransfer) {
        const names = [];
        for (let i = 0; i < e.dataTransfer.items.length; i++) {
          if (e.dataTransfer.items[i].kind === 'file') {
            const file = e.dataTransfer.items[i].getAsFile();
            if (file) {
              names.push(file.name);
            }
          }
        }
        ui.on_drag_over(names);
      }
    }
  });

  document.body.addEventListener('dragenter', e => {
    
  });

  document.body.addEventListener('dragleave', e => {
    ui.on_drag_leave();
  });

  document.addEventListener('drop', async e => {
    if (e.target === canvasNode) {
      e.preventDefault();
      e.stopPropagation();
      
      if (e.dataTransfer) {
        const files = [];
        for (let i = 0; i < e.dataTransfer.files.length; i++) {
          files.push(e.dataTransfer.files[i]);
        }
        const names = files.map(f => f.name);
        const contents = await Promise.all(files.map(async f => new Uint8Array(await f.arrayBuffer())));
        ui.on_file_dropped(names, contents);
      }
    }
  });

  function draw(canvas) {
    ui.on_draw(canvas);
    ui.get_view().surface.requestAnimationFrame(draw);
  }
  ui.get_view().surface.requestAnimationFrame(draw);

  return canvasNode;
}
