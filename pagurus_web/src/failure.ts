type Failure = {
  reason: string;
  backtrace: [{ file: string; line: number }];
};

export { Failure };
