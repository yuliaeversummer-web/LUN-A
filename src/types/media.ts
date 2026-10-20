export interface MediaCandidate { id: string; sourceId: string; name: string; eligible: boolean }
export interface MediaTrack { token: number; title: string; artist: string; album: string; artworkId: string | null }
export interface MediaCapabilities { play: boolean; pause: boolean; toggle: boolean; next: boolean; previous: boolean; seek: boolean }
export interface MediaTimeline { positionMs: number; confirmedPositionMs: number; durationMs: number; minSeekMs: number; maxSeekMs: number; startMs: number; estimated: boolean }
export interface MediaSnapshot {
 revision: number; status: 'loading' | 'ready' | 'paused' | 'unavailable' | 'error'; reason: string; message: string | null; warning: string | null
 selectedId: string | null; selectedSource: string | null; selectionMode: 'auto' | 'manual'; preferredSource: string | null
 candidates: MediaCandidate[]; track: MediaTrack | null; playback: string; capabilities: MediaCapabilities; timeline: MediaTimeline | null
 reaction: { sequence: number; kind: 'next' | 'error' } | null; commandError: {code: string; message: string} | null; persistenceError: string | null
}
