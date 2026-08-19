const fs = require('fs/promises');
const path = require('path');
const cheerio = require('cheerio');

module.exports = async function injectEnvToHtmlFiles(env, dir, ext='.html') {
  const publicEnv = Object
    .entries(env)
    .filter(([name, _]) => name.startsWith('PUBLIC_WEB_'));
  
  if (publicEnv.length < 1) {
    return;
  }
  
  const files = await fs.readdir(dir);
  for (const file of files) {
    const filepath = path.join(dir, file);
    const stat = await fs.stat(filepath);
    if(stat.isFile() && path.extname(file) === ext) {
      const fileHandle = await fs.open(filepath, 'r+');
      const contents = await fileHandle.readFile();
      const document = cheerio.load(contents);
      const bodyElement = document('body');

      publicEnv.forEach(([envName, envValue]) => {
        bodyElement.attr(`data-${envName.toLowerCase()}`, envValue);
      });
      await fileHandle.write(document.html(), 0);
      await fileHandle.close();
    }
  }
}