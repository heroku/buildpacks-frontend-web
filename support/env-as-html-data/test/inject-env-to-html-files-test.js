const assert = require('assert');
const { execFile } = require('child_process');
const { promisify } = require('util');
const fs = require('fs/promises');
const path = require('path');
const cheerio = require('cheerio');
const { createHash } = require('crypto');
const injectEnvToHtmlFiles = require('../lib/inject-env-to-html-files.js');

const execFileAsync = promisify(execFile);
const executablePath = path.resolve(__dirname, '../bin/env-as-html-data');

describe('injectEnvToHtmlFiles', function () {
  let testDir;

  beforeEach(async function () {
    testDir = await fs.mkdtemp('test_injectEnvToHtmlFiles_');
    await fs.cp('test/fixtures', testDir, {recursive: true});
  });

  afterEach(async function () {
    await fs.rm(testDir, {recursive: true});
  });

  async function loadDocument(relativePath) {
    return cheerio.load(await fs.readFile(path.join(testDir, relativePath)));
  }

  it('rewrites the default index document with env variables set in head data-* attributes', async function () {
    const testEnv = {
      PUBLIC_WEB_TEST: 'example value',
      PUBLIC_WEB_TEST_2: 'another value',
      SECRET_TEST: 'should not be exposed'
    };
    await injectEnvToHtmlFiles(testEnv, testDir);

    const indexDocument = await loadDocument('public/index.html');
    const pageDocument = await loadDocument('public/page-2.html');

    assert.equal(indexDocument('head').attr('data-public_web_test'), 'example value');
    assert.equal(indexDocument('head').attr('data-public_web_test_2'), 'another value');
    assert.equal(indexDocument('head').attr('data-secret_test'), null);
    assert.equal(indexDocument('body').attr('data-public_web_test'), undefined);
    assert.equal(pageDocument('head').attr('data-public_web_test'), undefined);
  });

  it('without env variables leaves files untouched', async function () {
    const testEnv = {};
    const expectedFiles = ['index.html', 'page-2.html'];
    const expectedFileHashes = [];
    const resultFileHashes = [];

    for (const filename of expectedFiles) {
      const fileHandle = await fs.open(path.join(testDir, 'public', filename), 'r+');
      const contents = await fileHandle.readFile();
      await fileHandle.close();
      expectedFileHashes.push(createHash('sha256').update(contents).digest('base64'));
    }

    await injectEnvToHtmlFiles(testEnv, testDir);

    for (const filename of expectedFiles) {
      const fileHandle = await fs.open(path.join(testDir, 'public', filename), 'r+');
      const contents = await fileHandle.readFile();
      await fileHandle.close();
      resultFileHashes.push(createHash('sha256').update(contents).digest('base64'));
    }

    assert.deepStrictEqual(resultFileHashes, expectedFileHashes);
  });

  it('uses project.toml to configure the document root and index document', async function () {
    await fs.writeFile(path.join(testDir, 'project.toml'), `
[com.heroku.static-web-server]
root = "configured-directory"
index = "index.html"
`);

    await execFileAsync(process.execPath, [executablePath], {
      cwd: testDir,
      env: {
        ...process.env,
        PUBLIC_WEB_TEST: 'configured directory value'
      }
    });

    const configuredDocument = await loadDocument('configured-directory/index.html');
    const defaultDocument = await loadDocument('public/index.html');

    assert.equal(configuredDocument('head').attr('data-public_web_test'), 'configured directory value');
    assert.equal(defaultDocument('head').attr('data-public_web_test'), undefined);
  });

  it('uses project.toml html_files with nested paths and globs', async function () {
    await fs.mkdir(path.join(testDir, 'public', 'subsection', 'nested'), {recursive: true});
    await fs.cp(path.join(testDir, 'public', 'index.html'), path.join(testDir, 'public', 'subsection', 'index.html'));
    await fs.cp(path.join(testDir, 'public', 'index.html'), path.join(testDir, 'public', 'subsection', 'nested', 'page.html'));
    await fs.writeFile(path.join(testDir, 'project.toml'), `
[com.heroku.static-web-server.runtime_config]
html_files = ["page-2.html", "subsection/**/*.html"]
`);

    await injectEnvToHtmlFiles({PUBLIC_WEB_TEST: 'nested value'}, testDir);

    assert.equal((await loadDocument('public/page-2.html'))('head').attr('data-public_web_test'), 'nested value');
    assert.equal((await loadDocument('public/subsection/index.html'))('head').attr('data-public_web_test'), 'nested value');
    assert.equal((await loadDocument('public/subsection/nested/page.html'))('head').attr('data-public_web_test'), 'nested value');
    assert.equal((await loadDocument('public/index.html'))('head').attr('data-public_web_test'), undefined);
  });

  it('truncates files after rewriting', async function () {
    const filePath = path.join(testDir, 'public', 'index.html');
    const original = await fs.readFile(filePath, 'utf8');
    await fs.writeFile(filePath, `${original}${'x'.repeat(1000)}`);

    await injectEnvToHtmlFiles({PUBLIC_WEB_TEST: 'shorter value'}, testDir);

    const rewritten = await fs.readFile(filePath, 'utf8');
    assert.equal(rewritten.endsWith('x'), false);
    assert.doesNotThrow(() => cheerio.load(rewritten));
  });

});
