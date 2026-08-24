const { transformHtml: transformHtmlInWasm } = require('../pkg/env_as_html_data_wasm.js');

class EnvAsHtmlDataError extends Error {
  constructor(message) {
    super(String(message));
    this.name = 'EnvAsHtmlDataError';
    this.code = this.message.startsWith('INVALID_ENVIRONMENT:')
      ? 'INVALID_ENVIRONMENT'
      : 'HTML_TRANSFORM_ERROR';
  }
}

function transformHtml(html, environment) {
  try {
    return transformHtmlInWasm(html, environment);
  } catch (error) {
    throw new EnvAsHtmlDataError(error.message ?? error);
  }
}

module.exports = { EnvAsHtmlDataError, transformHtml };
