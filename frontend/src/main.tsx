import React, {useState} from 'react';
import {createRoot} from 'react-dom/client';
import {verify} from './api';
import './style.css';
function App() {
 const [input,setInput]=useState(''); const [mode,setMode]=useState('conventional');
 const [busy,setBusy]=useState(false); const [message,setMessage]=useState('');
 return <main><p className="eyebrow">OPEN CREDENTIAL LEDGER · RESEARCH PROTOTYPE</p><h1>Verify trust.<br/>Keep credentials off-chain.</h1><p>Compare conventional issuer trust with an Ethereum issuer registry. Use synthetic credentials only.</p>
 <form onSubmit={async e=>{e.preventDefault();setBusy(true);setMessage('');try {const r=await verify(input,mode);setMessage(r.valid?'Signature valid and issuer currently trusted.':'Verification unsuccessful: signature invalid or issuer untrusted.');}catch(e){setMessage(e instanceof Error?e.message:'Verification unavailable.');}finally{setBusy(false);}}}>
 <label htmlFor="mode">Trust source</label><select id="mode" disabled={busy} value={mode} onChange={e=>setMode(e.target.value)}><option value="conventional">Conventional · PostgreSQL</option><option value="blockchain">Blockchain · issuer registry</option></select>
 <label htmlFor="credential">Signed credential envelope (JSON)</label><textarea id="credential" required disabled={busy} spellCheck={false} value={input} onChange={e=>setInput(e.target.value)} placeholder='{"issuer":"0x…","public_key":"…","payload_base64":"…","signature_base64":"…"}'/>
 <button disabled={busy}>{busy?'Verifying…':'Verify credential'}</button><p role="status" aria-live="polite">{message}</p></form><footer>Checks exact signed bytes and current key authorization. Academic claims, expiry, and credential-level revocation require separate policy checks.</footer></main>;
}
createRoot(document.getElementById('root')!).render(<React.StrictMode><App/></React.StrictMode>);
