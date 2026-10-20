import { test } from 'node:test'
import assert from 'node:assert/strict'
import { snapshotChannel } from '../src/services/snapshotChannel.ts'

const tick = () => new Promise(resolve => setTimeout(resolve, 5))
function fixture() {
  let receive = (_value: {revision:number}) => {}, visible = true, current = {revision:1}, stops = 0, reads = 0
  const applied: {revision:number;live:boolean}[] = [], errors: unknown[] = []
  const channel = snapshotChannel({read:async()=>{reads++;return current},subscribe:async fn=>{receive=fn;return()=>{stops++}},visible:async()=>visible,apply:(value,live)=>applied.push({...value,live}),error:cause=>errors.push(cause),intervalMs:100_000})
  return {channel,applied,errors,send:(revision:number)=>receive({revision}),setCurrent:(revision:number)=>{current={revision}},setVisible:(value:boolean)=>{visible=value;channel.visibility(value)},stops:()=>stops,reads:()=>reads}
}
test('old replies and duplicate events cannot roll state back or rerender',async()=>{
  const f=fixture();try{await tick();f.send(4);f.send(3);f.send(4);f.setCurrent(2);await f.channel.refresh();assert.deepEqual(f.applied.map(v=>v.revision),[1,4])}finally{f.channel.dispose()}
})
test('hidden window buffers only the latest snapshot; restore does not replay reactions',async()=>{
 const f=fixture();try{await tick();f.setVisible(false);f.send(2);f.send(3);assert.equal(f.applied.length,1);f.setCurrent(3);f.setVisible(true);await tick();assert.deepEqual(f.applied,[{revision:1,live:false},{revision:3,live:false}])}finally{f.channel.dispose()}
})
test('watchdog recovers a dropped event using a read, never a mutating command',async()=>{
 const f=fixture();try{await tick();f.setCurrent(8);await f.channel.refresh();assert.equal(f.applied.at(-1)?.revision,8);assert.equal(f.reads(),2)}finally{f.channel.dispose()}
})
test('dispose is idempotent and late events cannot update state',async()=>{
 const f=fixture();await tick();f.channel.dispose();f.channel.dispose();f.send(5);await f.channel.refresh();assert.equal(f.stops(),1);assert.equal(f.applied.length,1)
})
test('a subscription resolving after disposal is still unregistered',async()=>{
 let finish!:(stop:()=>void)=>void,stops=0;
 const channel=snapshotChannel({read:async()=>({revision:0}),subscribe:()=>new Promise<()=>void>(resolve=>{finish=resolve}),visible:async()=>true,apply:()=>{},error:()=>{}})
 channel.dispose();finish(()=>{stops++});await tick();assert.equal(stops,1)
})
test('a stalled subscription cannot leave the first snapshot loading forever',async()=>{
 const applied:number[]=[]
 const channel=snapshotChannel({read:async()=>({revision:3}),subscribe:()=>new Promise<()=>void>(()=>{}),visible:async()=>true,apply:v=>applied.push(v.revision),error:()=>{}})
 try{await tick();assert.deepEqual(applied,[3])}finally{channel.dispose()}
})
test('hide during an outstanding visibility query cannot reactivate updates',async()=>{
 let resolve!:(visible:boolean)=>void;const applied:number[]=[]
 const channel=snapshotChannel({read:async()=>({revision:1}),subscribe:async()=>()=>{},visible:()=>new Promise<boolean>(finish=>{resolve=finish}),apply:v=>applied.push(v.revision),error:()=>{}})
 try{channel.visibility(false);resolve(true);await tick();assert.deepEqual(applied,[])}finally{channel.dispose()}
})
test('failed initial native visibility query produces an error instead of endless loading',async()=>{
 const errors:unknown[]=[]
 const channel=snapshotChannel({read:async()=>({revision:1}),subscribe:async()=>()=>{},visible:async()=>{throw Error('visibility unavailable')},apply:()=>{},error:e=>errors.push(e)})
 try{await tick();assert.equal(errors.length,1)}finally{channel.dispose()}
})
test('an event arriving before a delayed initial read wins over that old reply',async()=>{
 let finish!:(value:{revision:number})=>void,receive!:(value:{revision:number})=>void;const applied:number[]=[]
 const channel=snapshotChannel({read:()=>new Promise<{revision:number}>(resolve=>{finish=resolve}),subscribe:async fn=>{receive=fn;return()=>{}},visible:async()=>true,apply:v=>applied.push(v.revision),error:()=>{}})
 try{await tick();receive({revision:9});finish({revision:1});await tick();assert.deepEqual(applied,[9])}finally{channel.dispose()}
})
test('a late subscription failure recovers with an unchanged authoritative revision',async()=>{
 let reject!:(error:Error)=>void,attempt=0;const applied:number[]=[],errors:unknown[]=[]
 const channel=snapshotChannel({read:async()=>({revision:1}),subscribe:()=>++attempt===1?new Promise<()=>void>((_,fail)=>{reject=fail}):Promise.resolve(()=>{}),visible:async()=>true,apply:v=>applied.push(v.revision),error:e=>errors.push(e)})
 try{await tick();reject(Error('listen failed after initial read'));await tick();await channel.refresh();assert.equal(errors.length,1);assert.deepEqual(applied,[1,1])}finally{channel.dispose()}
})
test('read timeout leaves loading, and the next successful read recovers even at the same revision',async()=>{
 let stalled=false;const applied:number[]=[],errors:unknown[]=[]
 const channel=snapshotChannel({read:()=>stalled?new Promise<{revision:number}>(()=>{}):Promise.resolve({revision:1}),subscribe:async()=>()=>{},visible:async()=>true,apply:v=>applied.push(v.revision),error:e=>errors.push(e),timeoutMs:10})
 try{await tick();stalled=true;await channel.refresh();assert.equal(errors.length,1);stalled=false;await channel.refresh();assert.deepEqual(applied,[1,1])}finally{channel.dispose()}
})
