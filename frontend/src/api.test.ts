import {afterEach,expect,it,vi} from 'vitest';
import {verify} from './api';
afterEach(()=>vi.unstubAllGlobals());
it('sends explicit selected mode',async()=>{const fetch=vi.fn().mockResolvedValue({ok:true,json:async()=>({valid:true})});vi.stubGlobal('fetch',fetch);expect((await verify('{"mode":"blockchain"}','conventional')).valid).toBe(true);expect(JSON.parse(fetch.mock.calls[0][1].body).mode).toBe('conventional');});
it('surfaces trust source outage',async()=>{vi.stubGlobal('fetch',vi.fn().mockResolvedValue({ok:false,status:503}));await expect(verify('{}','blockchain')).rejects.toThrow('unavailable');});
it('rejects nonobject input before network access',async()=>{await expect(verify('[]','conventional')).rejects.toThrow('JSON object');});
