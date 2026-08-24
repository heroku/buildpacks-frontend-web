const assert = require('node:assert/strict');
const { execFile } = require('node:child_process');
const { createHash } = require('node:crypto');
const fs = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
const { promisify } = require('node:util');

const { injectEnvToHtmlFiles, transformHtml } = require('..');

const HTML = '<html><head><title>Hello</title></head><body>Hello</body></html>';
const executablePath = path.resolve(__dirname, '../bin/env-as-html-data');
const fixturesPath = path.resolve(__dirname, 'fixtures');
const execFileAsync = promisify(execFile);

async function withFixtures(callback) {
  const appDirectory = await fs.mkdtemp(path.join(os.tmpdir(), 'env-as-html-data-'));
  await fs.cp(fixturesPath, appDirectory, { recursive: true });
  try {
    await callback(appDirectory);
  } finally {
    await fs.rm(appDirectory, { recursive: true, force: true });
  }
}

function headAttribute(html, name) {
  const head = html.match(/<head\b[^>]*>/)?.[0];
  return head?.match(new RegExp(`\\b${name}="([^"]*)"`))?.[1];
}

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

test('rewrites the default index document with public environment data', async () => {
  await withFixtures(async (appDirectory) => {
    await injectEnvToHtmlFiles({
      PUBLIC_WEB_TEST: 'example value',
      PUBLIC_WEB_TEST_2: 'another value',
      SECRET_TEST: 'should not be exposed',
    }, appDirectory);

    const indexHtml = await fs.readFile(path.join(appDirectory, 'public/index.html'), 'utf8');
    const pageHtml = await fs.readFile(path.join(appDirectory, 'public/page-2.html'), 'utf8');
    assert.equal(headAttribute(indexHtml, 'data-public_web_test'), 'example value');
    assert.equal(headAttribute(indexHtml, 'data-public_web_test_2'), 'another value');
    assert.equal(headAttribute(indexHtml, 'data-secret_test'), undefined);
    assert.equal(headAttribute(pageHtml, 'data-public_web_test'), undefined);
  });
});

test('leaves fixture files unchanged without public environment data', async () => {
  await withFixtures(async (appDirectory) => {
    const files = ['index.html', 'page-2.html'];
    const before = await Promise.all(files.map(async (file) => (
      createHash('sha256').update(await fs.readFile(path.join(appDirectory, 'public', file))).digest('base64')
    )));

    await injectEnvToHtmlFiles({}, appDirectory);

    const after = await Promise.all(files.map(async (file) => (
      createHash('sha256').update(await fs.readFile(path.join(appDirectory, 'public', file))).digest('base64')
    )));
    assert.deepEqual(after, before);
  });
});

test('CLI uses project.toml to configure the document root and index document', async () => {
  await withFixtures(async (appDirectory) => {
    await fs.writeFile(path.join(appDirectory, 'project.toml'), [
      '[com.heroku.static-web-server]',
      'root = "configured-directory"',
      'index = "index.html"',
    ].join('\n'));

    await execFileAsync(process.execPath, [executablePath], {
      cwd: appDirectory,
      env: { ...process.env, PUBLIC_WEB_TEST: 'configured directory value' },
    });

    const configuredHtml = await fs.readFile(path.join(appDirectory, 'configured-directory/index.html'), 'utf8');
    const defaultHtml = await fs.readFile(path.join(appDirectory, 'public/index.html'), 'utf8');
    assert.equal(headAttribute(configuredHtml, 'data-public_web_test'), 'configured directory value');
    assert.equal(headAttribute(defaultHtml, 'data-public_web_test'), undefined);
  });
});

test('uses project.toml html_files with nested paths and globs', async () => {
  await withFixtures(async (appDirectory) => {
    const publicDirectory = path.join(appDirectory, 'public');
    await fs.mkdir(path.join(publicDirectory, 'subsection/nested'), { recursive: true });
    await fs.cp(path.join(publicDirectory, 'index.html'), path.join(publicDirectory, 'subsection/index.html'));
    await fs.cp(path.join(publicDirectory, 'index.html'), path.join(publicDirectory, 'subsection/nested/page.html'));
    await fs.writeFile(path.join(appDirectory, 'project.toml'), [
      '[com.heroku.static-web-server.runtime_config]',
      'html_files = ["page-2.html", "subsection/**/*.html"]',
    ].join('\n'));

    await injectEnvToHtmlFiles({ PUBLIC_WEB_TEST: 'nested value' }, appDirectory);

    for (const file of ['page-2.html', 'subsection/index.html', 'subsection/nested/page.html']) {
      assert.equal(headAttribute(await fs.readFile(path.join(publicDirectory, file), 'utf8'), 'data-public_web_test'), 'nested value');
    }
    assert.equal(headAttribute(await fs.readFile(path.join(publicDirectory, 'index.html'), 'utf8'), 'data-public_web_test'), undefined);
  });
});

test('truncates files after rewriting', async () => {
  await withFixtures(async (appDirectory) => {
    const filePath = path.join(appDirectory, 'public/index.html');
    await fs.appendFile(filePath, 'x'.repeat(1000));

    await injectEnvToHtmlFiles({ PUBLIC_WEB_TEST: 'shorter value' }, appDirectory);

    const rewritten = await fs.readFile(filePath, 'utf8');
    assert.equal(rewritten.endsWith('x'), false);
    assert.match(rewritten, /<html\b/);
    assert.equal(headAttribute(rewritten, 'data-public_web_test'), 'shorter value');
  });
});
