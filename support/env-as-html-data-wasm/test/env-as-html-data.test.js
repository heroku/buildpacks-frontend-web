const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');

const { injectEnvToHtmlFiles, transformHtml } = require('..');

const HTML = '<html><head><title>Hello</title></head><body>Hello</body></html>';

test('transformHtml injects public environment data into the head', () => {
  assert.equal(
    transformHtml(HTML, {
      PUBLIC_WEB_API_URL: 'https://api.example.com',
      NOT_PUBLIC: 'not exposed',
    }),
    '<html><head data-public_web_api_url="https://api.example.com"><title>Hello</title></head><body>Hello</body></html>',
  );
});

test('transformHtml returns HTML unchanged without public environment data', () => {
  assert.equal(transformHtml(HTML, { NOT_PUBLIC: 'not exposed' }), HTML);
});

test('transformHtml rejects non-string environment values', () => {
  assert.throws(
    () => transformHtml(HTML, { PUBLIC_WEB_ENABLED: true }),
    { name: 'EnvAsHtmlDataError', code: 'INVALID_ENVIRONMENT' },
  );
});

test('injectEnvToHtmlFiles uses runtime configuration and glob target files', async () => {
  const appDirectory = await fs.mkdtemp(path.join(os.tmpdir(), 'env-as-html-data-wasm-'));
  const publicDirectory = path.join(appDirectory, 'web');
  await fs.mkdir(path.join(publicDirectory, 'nested'), { recursive: true });
  await fs.writeFile(path.join(appDirectory, 'project.toml'), [
    '[com.heroku.static-web-server]',
    'root = "web"',
    '',
    '[com.heroku.static-web-server.runtime_config]',
    'html_files = ["**/*.html"]',
  ].join('\n'));
  await fs.writeFile(path.join(publicDirectory, 'index.html'), HTML);
  await fs.writeFile(path.join(publicDirectory, 'nested', 'page.html'), HTML);

  await injectEnvToHtmlFiles({ PUBLIC_WEB_RELEASE_VERSION: 'v1' }, appDirectory);

  for (const htmlFile of ['index.html', path.join('nested', 'page.html')]) {
    const html = await fs.readFile(path.join(publicDirectory, htmlFile), 'utf8');
    assert.match(html, /data-public_web_release_version="v1"/);
  }
});
