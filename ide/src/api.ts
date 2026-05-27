export type RunRequest = {
  source: string;
};

export type IdeDiagnostic = {
  message: string;
  line?: number | null;
  column?: number | null;
  start?: number | null;
  end?: number | null;
};

export type RunResponse = {
  ok: boolean;
  output: string[];
  diagnostics: IdeDiagnostic[];
};

export async function runPepsSource(source: string): Promise<RunResponse> {
  const request: RunRequest = { source };
  const response = await fetch("/api/run", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify(request)
  });

  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }

  return await response.json();
}
