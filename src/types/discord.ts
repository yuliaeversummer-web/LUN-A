import type { AudioSession } from './audio'
export interface DiscordPreferences {client:'stable'|'ptb'|'canary';enabled:boolean;confirmed:boolean;mute:string;deafen:string}
export interface DiscordAttempt {sequence:number;action:'mute'|'deafen';status:'sent'|'not_sent';message:string;timestampMs:number;code:string|null}
export interface DiscordSnapshot {revision:number;configurationRevision:number;preferences:DiscordPreferences;clients:{variant:string;installed:boolean;running:boolean;pids:number[];privilegeCompatible:boolean}[];status:string;reason:string;message:string;canSend:boolean;mode:'hotkeys';mute:boolean|null;deafen:boolean|null;audioSessions:AudioSession[];lastAttempt:DiscordAttempt|null;persistenceError:string|null}
