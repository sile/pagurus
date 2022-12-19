interface Failure {
  code?: number;
  message?: string;
  backtrace?: [{ file: string; line: number }];
}

export { Failure };
