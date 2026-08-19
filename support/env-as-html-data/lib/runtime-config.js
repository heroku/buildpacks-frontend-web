const fs = require('fs/promises');
const path = require('path');
const toml = require('toml');

const DEFAULT_DOCUMENT_ROOT = 'public';
const DEFAULT_INDEX_DOCUMENT = 'index.html';

async function loadRuntimeConfig(appDir) {
  let contents;
  try {
    contents = await fs.readFile(path.join(appDir, 'project.toml'), 'utf8');
  } catch (error) {
    if (error.code === 'ENOENT') {
      contents = '';
    } else {
      throw error;
    }
  }

  const webServerConfig = toml.parse(contents).com?.heroku?.['static-web-server'] || {};
  const runtimeConfig = webServerConfig.runtime_config || {};
  return {
    root: webServerConfig.root || DEFAULT_DOCUMENT_ROOT,
    index: webServerConfig.index || DEFAULT_INDEX_DOCUMENT,
    enabled: runtimeConfig.enabled ?? true,
    htmlFiles: runtimeConfig.html_files
  };
}

module.exports = { loadRuntimeConfig };
