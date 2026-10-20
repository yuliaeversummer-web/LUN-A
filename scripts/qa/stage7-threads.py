"""Read-only thread names for the supplied LUNÉA process. No input/window APIs."""
import ctypes as c
from ctypes import wintypes as w
import json
import sys

k = c.WinDLL('kernel32', use_last_error=True)
class Entry(c.Structure):
    _fields_ = [('size',w.DWORD),('usage',w.DWORD),('tid',w.DWORD),('owner',w.DWORD),('priority',w.LONG),('delta',w.LONG),('flags',w.DWORD)]
k.CreateToolhelp32Snapshot.argtypes = [w.DWORD,w.DWORD]
k.CreateToolhelp32Snapshot.restype = w.HANDLE
k.Thread32First.argtypes = [w.HANDLE,c.POINTER(Entry)]
k.Thread32Next.argtypes = k.Thread32First.argtypes
k.OpenThread.argtypes = [w.DWORD,w.BOOL,w.DWORD]
k.OpenThread.restype = w.HANDLE
k.GetThreadDescription.argtypes = [w.HANDLE,c.POINTER(c.c_void_p)]
k.GetThreadDescription.restype = c.c_long
k.CloseHandle.argtypes = [w.HANDLE]
k.LocalFree.argtypes = [c.c_void_p]
k.LocalFree.restype = c.c_void_p
pid = int(sys.argv[1])
snapshot = k.CreateToolhelp32Snapshot(4,0)
entry = Entry(size=c.sizeof(Entry))
names, failures = [], []
try:
    more = k.Thread32First(snapshot,c.byref(entry))
    while more:
        if entry.owner == pid:
            handle = k.OpenThread(0x0800,False,entry.tid)
            if handle:
                text = c.c_void_p()
                try:
                    status = k.GetThreadDescription(handle,c.byref(text))
                    if status >= 0 and text.value:
                        name = c.wstring_at(text.value)
                        if name: names.append(name)
                    elif status < 0: failures.append(status)
                finally:
                    if text.value: k.LocalFree(text)
                    k.CloseHandle(handle)
        more = k.Thread32Next(snapshot,c.byref(entry))
finally:
    k.CloseHandle(snapshot)
print(json.dumps({'pid':pid,'workers':{name:names.count(name) for name in ['lunea-core-audio','lunea-gsmtc','lunea-discord']},'namedThreads':len(names),'queryFailures':failures}))
