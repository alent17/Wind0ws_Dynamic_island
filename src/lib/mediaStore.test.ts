import { beforeEach, expect, it, vi } from 'vitest';
const api = vi.hoisted(() => ({listen:vi.fn(), getMediaInfo:vi.fn()}));
vi.mock('@tauri-apps/api/event',()=>({listen:api.listen}));
vi.mock('$lib/api/media',()=>({mediaApi:{getMediaInfo:api.getMediaInfo}}));
beforeEach(()=>{vi.resetModules();vi.clearAllMocks();api.getMediaInfo.mockRejectedValue(new Error('offline'));});
it('shares one listener and releases only after the final consumer', async()=>{
  const dispose=vi.fn();api.listen.mockResolvedValue(dispose);
  const {connectMedia}=await import('./mediaStore');
  const first=await connectMedia(),second=await connectMedia();
  expect(api.listen).toHaveBeenCalledTimes(1);
  first();first();expect(dispose).not.toHaveBeenCalled();
  second();expect(dispose).toHaveBeenCalledTimes(1);
});
it('releases a listener that resolves after its page is destroyed',async()=>{
  let resolve!:(fn:()=>void)=>void;
  api.listen.mockReturnValue(new Promise(r=>resolve=r));
  const {connectMedia}=await import('./mediaStore');
  const release=await connectMedia();release();
  const dispose=vi.fn();resolve(dispose);await Promise.resolve();
  expect(dispose).toHaveBeenCalledTimes(1);
});
