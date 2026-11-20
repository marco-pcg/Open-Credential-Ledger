import {afterEach,expect,it,vi} from 'vitest';
import {verify, verificationMessage, type Outcome} from './api';
afterEach(()=>vi.unstubAllGlobals());
it('sends explicit selected mode',async()=>{const fetch=vi.fn().mockResolvedValue({ok:true,json:async()=>({valid:true})});vi.stubGlobal('fetch',fetch);expect((await verify('{"mode":"blockchain"}','conventional')).valid).toBe(true);expect(JSON.parse(fetch.mock.calls[0][1].body).mode).toBe('conventional');});
it('surfaces trust source outage',async()=>{vi.stubGlobal('fetch',vi.fn().mockResolvedValue({ok:false,status:503}));await expect(verify('{}','blockchain')).rejects.toThrow('unavailable');});
it('rejects nonobject input before network access',async()=>{await expect(verify('[]','conventional')).rejects.toThrow('JSON object');});

const outcome = (reason: Outcome['reason']): Outcome => ({
  valid:reason==='verified', signature_valid:reason!=='invalid_signature',
  claims_valid:['verified','untrusted_issuer','revoked_credential'].includes(reason),
  issuer_trusted:reason==='untrusted_issuer'?false:['verified','revoked_credential'].includes(reason)?true:null,
  credential_revoked:reason==='verified'?false:reason==='revoked_credential'?true:null,
  reason,policy_version:'ocl.academic.v1',evaluated_at:1800000000,
});
it('explains successful policy verification without claiming award authenticity',()=>{
  expect(verificationMessage(outcome('verified'))).toContain('credential policy');
});
it.each([
  ['expired','expired'], ['not_yet_valid','not yet valid'], ['issuer_mismatch','does not match'],
  ['invalid_claims','required credential format'], ['unsupported_version','not supported'],
  ['invalid_dates','inconsistent dates'], ['invalid_signature','signature does not match'],
  ['untrusted_issuer','not currently authorized'], ['revoked_credential','has been revoked'],
] as const)('explains %s decisions', (reason, text)=>{
  expect(verificationMessage(outcome(reason))).toContain(text);
});
