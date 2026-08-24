export type PublicEnvironment = Readonly<Record<string, string>>;

export class EnvAsHtmlDataError extends Error {
  code: "INVALID_ENVIRONMENT" | "HTML_TRANSFORM_ERROR";
}

export function transformHtml(html: string, environment: PublicEnvironment): string;

export function injectEnvToHtmlFiles(
  environment: PublicEnvironment,
  appDirectory: string,
): Promise<void>;
