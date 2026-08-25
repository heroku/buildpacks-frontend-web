const { EnvAsHtmlDataError, transformHtml } = require('./lib/transform-html.js');
const { injectEnvToHtmlFiles } = require('./lib/inject-env-to-html-files.js');

module.exports = {
  EnvAsHtmlDataError,
  injectEnvToHtmlFiles,
  transformHtml,
};
