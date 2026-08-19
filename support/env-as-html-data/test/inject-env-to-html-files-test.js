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

  it('rewrites files with env variables set in body data-* attributes', async function () {
    const testEnv = {
      PUBLIC_WEB_TEST: 'example value',
      PUBLIC_WEB_TEST_2: 'another value',
      SECRET_TEST: 'should not be exposed'
    };
    await injectEnvToHtmlFiles(testEnv, path.join(testDir, 'public'));

    const expectedFiles = ['index.html', 'page-2.html'];
    for (const filename of expectedFiles) {
      const fileHandle = await fs.open(path.join(testDir, 'public', filename), 'r+');
      const contents = await fileHandle.readFile();
      const document = cheerio.load(contents);
      await fileHandle.close();
      const bodyElement = document('body');
      const bodyAttrs = bodyElement.attr();

      assert.equal(bodyAttrs['data-public_web_test'], 'example value', `PUBLIC_WEB_TEST env var is set incorrectly in ${filename}`);
      assert.equal(bodyAttrs['data-public_web_test_2'], 'another value', `PUBLIC_WEB_TEST_2 env var is set incorrectly in ${filename}`);
      assert.equal(bodyAttrs['data-secret_test'], null, `SECRET_TEST env var should not be set in ${filename}, because its name is not prefixed with PUBLIC_WEB_`);
    }
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

    await injectEnvToHtmlFiles(testEnv, path.join(testDir, 'public'));

    for (const filename of expectedFiles) {
      const fileHandle = await fs.open(path.join(testDir, 'public', filename), 'r+');
      const contents = await fileHandle.readFile();
      await fileHandle.close();
      resultFileHashes.push(createHash('sha256').update(contents).digest('base64'));
    }

    assert.deepStrictEqual(resultFileHashes, expectedFileHashes);
  });

  it('uses ENV_AS_HTML_DATA_DIR instead of the default public directory', async function () {
    await execFileAsync(process.execPath, [executablePath], {
      cwd: testDir,
      env: {
        ...process.env,
        ENV_AS_HTML_DATA_DIR: 'configured-directory',
        PUBLIC_WEB_TEST: 'configured directory value'
      }
    });

    const configuredDocument = cheerio.load(
      await fs.readFile(path.join(testDir, 'configured-directory', 'index.html'))
    );
    const defaultDocument = cheerio.load(
      await fs.readFile(path.join(testDir, 'public', 'index.html'))
    );

    assert.equal(configuredDocument('body').attr('data-public_web_test'), 'configured directory value');
    assert.equal(defaultDocument('body').attr('data-public_web_test'), undefined);
  });

  it('uses ENV_AS_HTML_DATA_FILE_EXT to select files to process', async function () {
    await execFileAsync(process.execPath, [executablePath], {
      cwd: testDir,
      env: {
        ...process.env,
        ENV_AS_HTML_DATA_DIR: 'configured-extension',
        ENV_AS_HTML_DATA_FILE_EXT: '.template',
        PUBLIC_WEB_TEST: 'configured extension value'
      }
    });

    const matchingDocument = cheerio.load(
      await fs.readFile(path.join(testDir, 'configured-extension', 'index.template'))
    );
    const nonMatchingDocument = cheerio.load(
      await fs.readFile(path.join(testDir, 'configured-extension', 'index.html'))
    );

    assert.equal(matchingDocument('body').attr('data-public_web_test'), 'configured extension value');
    assert.equal(nonMatchingDocument('body').attr('data-public_web_test'), undefined);
  });
});
