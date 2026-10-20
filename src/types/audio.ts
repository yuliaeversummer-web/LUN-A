export interface AudioDevice { id: string; name: string; state: 'active' | 'disabled' | 'unplugged' | 'notPresent'; defaultConsole: boolean; defaultMultimedia: boolean; defaultCommunications: boolean }
export interface AudioEndpoint { deviceId: string; name: string; volume: number; muted: boolean }
export interface AudioSession { id: string; deviceId: string; deviceName: string; pid: number; name: string; icon: string | null; volume: number; muted: boolean; state: 'active' | 'inactive'; system: boolean }
export interface AudioSnapshot { revision: number; status: 'loading' | 'ready' | 'unavailable' | 'error'; endpoint: AudioEndpoint | null; sessions: AudioSession[]; outputs: AudioDevice[]; inputs: AudioDevice[]; message: string | null; warning: string | null }
