// Renders the design boards (../boards/*.dc.html) to static pages + reference PNGs, and diffs a running app screen against them.
//   node docs/design-refs/tools/design-ref.mjs render [Board ...]
//   node docs/design-refs/tools/design-ref.mjs diff <Board> <app URL> [--storage state.json] [--wait <css selector>]
// Needs Playwright: PLAYWRIGHT=/home/dbsrh/qa-tools/node_modules/playwright/index.mjs (or `playwright` resolvable from here).
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const BOARDS = path.join(ROOT, 'boards');
const ASSETS = path.join(BOARDS, 'assets');
const OUT = path.join(ROOT, 'out');
const pw = await import(process.env.PLAYWRIGHT ? pathToFileURL(process.env.PLAYWRIGHT).href : 'playwright');

const assetFile = Object.fromEntries(fs.readdirSync(ASSETS).map((f) => [f.split('.')[0], f]));
const evalIn = (expr, scope) => new Function('s', `with (s) { return (${expr}); }`)(scope);
const fill = (str, scope) => str.replace(/\{\{\s*([^}]+?)\s*\}\}/g, (_, e) => { const v = evalIn(e, scope); return v == null ? '' : String(v); });

// Expands <sc-for list="{{expr}}" as="x"> loops (nested ok), then fills {{holes}} with the loop/renderVals scope.
function expand(str, scope) {
  const open = str.search(/<sc-for\b/);
  if (open < 0) return fill(str, scope);
  const tagEnd = str.indexOf('>', open);
  const tag = str.slice(open, tagEnd + 1);
  let depth = 1, i = tagEnd + 1, close = -1;
  const re = /<sc-for\b|<\/sc-for>/g;
  re.lastIndex = i;
  for (let m; (m = re.exec(str));) { depth += m[0] === '</sc-for>' ? -1 : 1; if (!depth) { close = m.index; break; } }
  if (close < 0) throw new Error('unclosed <sc-for>');
  const list = evalIn(tag.match(/list="\{\{\s*([^}]+?)\s*\}\}"/)[1], scope) || [];
  const as = tag.match(/as="(\w+)"/)[1];
  const inner = str.slice(tagEnd + 1, close);
  return fill(str.slice(0, open), scope) + list.map((it) => expand(inner, { ...scope, [as]: it })).join('') + expand(str.slice(close + '</sc-for>'.length), scope);
}

function renderBoard(name) {
  const src = fs.readFileSync(path.join(BOARDS, `${name}.dc.html`), 'utf8');
  const head = (src.match(/<helmet>([\s\S]*?)<\/helmet>/) || [, ''])[1];
  let body = src.match(/<x-dc>([\s\S]*?)<\/x-dc>/)[1].replace(/<helmet>[\s\S]*?<\/helmet>/, '');
  const script = src.match(/<script type="text\/x-dc" data-dc-script data-props='([^']*)'>([\s\S]*?)<\/script>/);
  const preview = JSON.parse(script[1]).$preview;
  const vals = new Function('DCLogic', `${script[2]}\nreturn new Component().renderVals();`)(class { constructor() { this.props = {}; } });
  body = expand(body, vals);
  const heads = [head];
  body = body.replace(/<dc-import name="(\w+)"[^>]*><\/dc-import>/g, (_, n) => { const r = renderBoard(n); heads.push(r.head); return r.body; });
  return { head: heads.join('\n'), body, preview };
}

function toHtml(name) {
  const { head, body, preview } = renderBoard(name);
  const html = `<!doctype html><html lang="ko"><head><meta charset="utf-8"><title>${name}</title><style>html{background:#0A0C0F}</style>${head}</head><body>${body}</body></html>`
    // inlined as data URLs: CSS mask images (the wave icon) don't load from file:// pages
    .replace(/\/_blob\/([0-9a-f]{32})/g, (_, id) => {
      const f = assetFile[id];
      if (!f) throw new Error(`${name}: missing asset ${id}`);
      const mime = { png: 'image/png', jpg: 'image/jpeg', webp: 'image/webp' }[f.split('.').pop()];
      return `data:${mime};base64,${fs.readFileSync(path.join(ASSETS, f)).toString('base64')}`;
    });
  return { html, preview };
}

async function shot(page, url, preview, file, wait) {
  await page.setViewportSize({ width: preview.width, height: preview.height });
  await page.goto(url, { waitUntil: 'networkidle' });
  if (wait) await page.waitForSelector(wait);
  await page.evaluate(() => document.fonts.ready);
  await page.screenshot({ path: file, fullPage: true, animations: 'disabled' });
}

const [cmd, ...args] = process.argv.slice(2);
fs.mkdirSync(OUT, { recursive: true });
const browser = await pw.chromium.launch();
try {
  if (cmd === 'render') {
    const names = args.length ? args : fs.readdirSync(BOARDS).filter((f) => f.endsWith('.dc.html')).map((f) => f.slice(0, -8));
    const page = await browser.newPage();
    for (const n of names) {
      const { html, preview } = toHtml(n);
      const file = path.join(OUT, `${n}.html`);
      fs.writeFileSync(file, html);
      await shot(page, pathToFileURL(file).href, preview, path.join(OUT, `${n}.png`));
      console.log(`${n}  ${preview.width}x${preview.height}`);
    }
  } else if (cmd === 'diff') {
    const [name, url] = args;
    const opt = (k) => { const i = args.indexOf(k); return i < 0 ? undefined : args[i + 1]; };
    if (!name || !url) throw new Error('usage: diff <Board> <app URL> [--storage state.json] [--wait selector]');
    const { html, preview } = toHtml(name);
    const refHtml = path.join(OUT, `${name}.html`);
    fs.writeFileSync(refHtml, html);
    const refPng = path.join(OUT, `${name}.png`), appPng = path.join(OUT, `${name}.app.png`);
    await shot(await browser.newPage(), pathToFileURL(refHtml).href, preview, refPng);
    const ctx = await browser.newContext(opt('--storage') ? { storageState: opt('--storage') } : {});
    await shot(await ctx.newPage(), url, preview, appPng, opt('--wait'));
    // Pixel compare in a blank page: panels = design | app | diff (red = differs by >48 summed RGB).
    const page = await browser.newPage();
    const b64 = (f) => `data:image/png;base64,${fs.readFileSync(f).toString('base64')}`;
    const r = await page.evaluate(async ([a, b]) => {
      const load = (s) => new Promise((ok) => { const i = new Image(); i.onload = () => ok(i); i.src = s; });
      const [ia, ib] = await Promise.all([load(a), load(b)]);
      const w = Math.max(ia.width, ib.width), h = Math.max(ia.height, ib.height);
      const px = (img) => { const c = new OffscreenCanvas(w, h), x = c.getContext('2d'); x.fillStyle = '#fff'; x.fillRect(0, 0, w, h); x.drawImage(img, 0, 0); return x.getImageData(0, 0, w, h); };
      const da = px(ia), db = px(ib), dd = new ImageData(w, h);
      let bad = 0;
      for (let i = 0; i < da.data.length; i += 4) {
        const d = Math.abs(da.data[i] - db.data[i]) + Math.abs(da.data[i + 1] - db.data[i + 1]) + Math.abs(da.data[i + 2] - db.data[i + 2]);
        const g = (da.data[i] + da.data[i + 1] + da.data[i + 2]) / 12;
        if (d > 48) { bad++; dd.data.set([255, 40, 60, 255], i); } else dd.data.set([g, g, g, 255], i);
      }
      const out = new OffscreenCanvas(w * 3 + 32, h), o = out.getContext('2d');
      o.fillStyle = '#ff00ff'; o.fillRect(0, 0, out.width, h);
      o.drawImage(ia, 0, 0); o.drawImage(ib, w + 16, 0); o.putImageData(dd, w * 2 + 32, 0);
      const blob = await out.convertToBlob({ type: 'image/png' });
      const bytes = new Uint8Array(await blob.arrayBuffer());
      let s = ''; for (let i = 0; i < bytes.length; i += 0x8000) s += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
      return { pct: (bad / (w * h)) * 100, sizes: [ia.width, ia.height, ib.width, ib.height], png: btoa(s) };
    }, [b64(refPng), b64(appPng)]);
    const diffPng = path.join(OUT, `${name}.diff.png`);
    fs.writeFileSync(diffPng, Buffer.from(r.png, 'base64'));
    console.log(`${name}: ${r.pct.toFixed(2)}% pixels differ  design ${r.sizes[0]}x${r.sizes[1]}  app ${r.sizes[2]}x${r.sizes[3]}\n${diffPng}`);
  } else {
    throw new Error('usage: render [Board ...] | diff <Board> <app URL> [--storage state.json] [--wait selector]');
  }
} finally {
  await browser.close();
}
