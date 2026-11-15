export type Outcome = {valid:boolean; signature_valid:boolean; issuer_trusted:boolean};
export async function verify(input: string, mode: string): Promise<Outcome> {
  const credential: unknown = JSON.parse(input);
  if (!credential || typeof credential !== 'object' || Array.isArray(credential)) throw new Error('Enter a credential JSON object.');
  const response = await fetch('/api/verify', {method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({...credential,mode}),signal:AbortSignal.timeout(12000)});
  if (!response.ok) throw new Error(response.status === 503 ? 'Trust source unavailable. No validity decision was made.' : `Verification failed (${response.status}). Check the credential format.`);
  return response.json();
}
