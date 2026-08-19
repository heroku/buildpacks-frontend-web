const fs = require('fs/promises');
const path = require('path');
const cheerio = require('cheerio');
const { glob } = require('glob');
const { loadRuntimeConfig } = require('./runtime-config.js');

module.exports = async function injectEnvToHtmlFiles(env, appDir) {
  const publicEnv = Object
    .entries(env)
    .filter(([name, _]) => name.startsWith('PUBLIC_WEB_'));

  if (publicEnv.length < 1) {
    return;
  }

  const runtimeConfig = await loadRuntimeConfig(appDir);
  if (!runtimeConfig.enabled) {
    return;
  }

  const documentRoot = path.join(appDir, runtimeConfig.root);
  const htmlFiles = runtimeConfig.htmlFiles || [runtimeConfig.index];
  const filePaths = (await Promise.all(htmlFiles.map(async (htmlFile) => {
    if (htmlFile.includes('*')) {
      return glob(htmlFile, {cwd: documentRoot, nodir: true});
    }
    return [htmlFile];
  }))).flat();

  for (const htmlFile of filePaths) {
    const filepath = path.join(documentRoot, htmlFile);
    const fileHandle = await fs.open(filepath, 'r+');
    try {
      const contents = await fileHandle.readFile();
      const document = cheerio.load(contents);
      const headElement = document('head');

      publicEnv.forEach(([envName, envValue]) => {
        headElement.attr(`data-${envName.toLowerCase()}`, envValue);
      });
      await fileHandle.truncate(0);
      await fileHandle.write(document.html(), 0);
    } finally {
      await fileHandle.close();
    }
  }
}
