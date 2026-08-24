const fs = require('node:fs/promises');
const path = require('node:path');

const { glob } = require('glob');
const { transformHtml } = require('./transform-html.js');
const { loadRuntimeConfig } = require('./runtime-config.js');

module.exports.injectEnvToHtmlFiles = async function injectEnvToHtmlFiles(environment, appDirectory) {
  const publicEnvironment = Object.fromEntries(
    Object.entries(environment).filter(([name]) => (
      name.startsWith('PUBLIC_WEB_') || name.startsWith('public_web_')
    )),
  );

  if (Object.keys(publicEnvironment).length === 0) {
    return;
  }

  const runtimeConfig = await loadRuntimeConfig(appDirectory);
  if (!runtimeConfig.enabled) {
    return;
  }

  const documentRoot = path.join(appDirectory, runtimeConfig.root);
  const htmlFiles = runtimeConfig.htmlFiles ?? [runtimeConfig.index];
  const filePaths = (await Promise.all(htmlFiles.map((htmlFile) => (
    htmlFile.includes('*')
      ? glob(htmlFile, { cwd: documentRoot, nodir: true })
      : [htmlFile]
  )))).flat();

  for (const htmlFile of filePaths) {
    const filePath = path.join(documentRoot, htmlFile);
    const html = await fs.readFile(filePath, 'utf8');
    await fs.writeFile(filePath, transformHtml(html, publicEnvironment));
  }
};
