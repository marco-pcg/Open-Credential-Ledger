export type Decision = 'verified' | 'invalid_signature' | 'invalid_claims' | 'unsupported_version' |
  'issuer_mismatch' | 'invalid_dates' | 'not_yet_valid' | 'expired' | 'untrusted_issuer';
export type Outcome = {
  valid: boolean;
  signature_valid: boolean;
  claims_valid: boolean;
  issuer_trusted: boolean | null;
  reason: Decision;
  policy_version: string;
  evaluated_at: number;
};
export function verificationMessage(result: Outcome): string {
  const messages: Record<Decision, string> = {
    verified: 'Signature, credential policy, and current issuer authorization passed.',
    invalid_signature: 'The signature does not match the signed credential bytes.',
    invalid_claims: 'The signed credential does not match the required credential format.',
    unsupported_version: 'This credential profile version is not supported.',
    issuer_mismatch: 'The signed issuer does not match the issuer in the envelope.',
    invalid_dates: 'The credential contains invalid or inconsistent dates.',
    not_yet_valid: 'The credential is not yet valid.',
    expired: 'The credential has expired.',
    untrusted_issuer: 'The signature and claims passed, but the issuer key is not currently authorized.',
  };
  return messages[result.reason] ?? 'Verification unsuccessful.';
}
export async function verify(input: string, mode: string): Promise<Outcome> {
  const credential: unknown = JSON.parse(input);
  if (!credential || typeof credential !== 'object' || Array.isArray(credential)) throw new Error('Enter a credential JSON object.');
  const response = await fetch('/api/verify', {method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({...credential,mode}),signal:AbortSignal.timeout(12000)});
  if (!response.ok) throw new Error(response.status === 503 ? 'Verification unavailable. No validity decision was made.' : `Verification failed (${response.status}). Check the credential format.`);
  return response.json();
}
