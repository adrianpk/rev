#!/usr/bin/python3
# Controlled RPC peer. It never performs a provider request or reads credentials.
import json, os, sys, time
from pathlib import Path
root = Path(__file__).parent
mode = (root / 'mode').read_text()
if '--version' in sys.argv:
    (root / 'version-pid').write_text(str(os.getpid()))
    if mode == 'version_hang': time.sleep(20)
    print('9.0.0' if mode == 'version' else '1.3.0', flush=True)
    sys.exit(0)
(root / 'pid').write_text(str(os.getpid()))
(root / 'args').write_text(json.dumps(sys.argv[1:]))
(root / 'cwd').write_text(os.getcwd())
(root / 'rpc-started').touch()
def emit(v):
    sys.stdout.write(json.dumps(v, ensure_ascii=False) + '\n')
    sys.stdout.flush()
def response(r, data=None, success=True):
    emit(dict(type='response', id=r['id'], command=r['type'], success=success, data=data))
def final(text='A selection explanation.\u2028With Unicode.'):
    emit(dict(type='message_end', message=dict(role='assistant', stopReason='stop', content=[dict(type='text',text=json.dumps(dict(version=1,text=text),ensure_ascii=False))])))
def settle(): emit(dict(type='agent_settled',aborted=False))
for line in sys.stdin:
    r = json.loads(line)
    with (root/'requests').open('a') as log: log.write(json.dumps(r)+'\n')
    kind = r['type']
    if kind == 'get_state':
        response(r, dict(isStreaming=False, isCompacting=False, messageCount=1 if mode=='history' else 0, pendingMessageCount=0,model=dict(provider='fixture',id='controlled')))
    elif kind == 'prompt':
        (root/'prompt').write_text(r['message'])
        if mode == 'rejected':
            response(r, success=False)
            continue
        if mode == 'correlation':
            r['id']='another-operation'
        if mode != 'late_accept': response(r,dict(disposition=mode if mode in ['handled','queued'] else 'started'))
        if mode == 'duplicate': response(r,dict(disposition='started'))
        if mode == 'empty_settled': settle();continue
        if mode == 'truncated':
            emit(dict(type='message_end',message=dict(role='assistant',stopReason='length',content=[])));settle();continue
        if mode == 'stderr':
            sys.stderr.write('diagnostic-only\n' * 30000);sys.stderr.flush()
        if mode == 'malformed':
            sys.stdout.write('{invalid}\n');sys.stdout.flush();continue
        if mode == 'no_lf':
            sys.stdout.write('x' * (2*1024*1024+1));sys.stdout.flush();continue
        if mode == 'tool':
            emit(dict(type='tool_execution_start',toolName='bash'));continue
        if mode == 'turns':
            for _ in range(9): emit(dict(type='turn_start'))
            continue
        if mode == 'auth':
            emit(dict(type='message_end',message=dict(role='assistant',stopReason='error',errorMessage='SECRET-FIXTURE-MUST-NOT-LEAK',content=[])));settle();continue
        if mode in ['hold','late_cancel','stubborn','eof','aborted','invalid','oversize','handled']:
            if mode == 'eof': sys.exit(0)
            if mode == 'aborted': emit(dict(type='agent_settled',aborted=True))
            if mode == 'invalid': emit(dict(type='message_end',message=dict(role='assistant',stopReason='stop',content=[dict(type='text',text='{"version":1,"text":"bad","path":"/tmp/other"}')])));settle()
            if mode == 'oversize': final('x'*500);settle()
            continue
        final();emit(dict(type='agent_end',messages=[]))
        if mode == 'settle_gate':
            (root/'partial').touch()
            while not (root/'release').exists(): time.sleep(.005)
        settle()
        if mode == 'late_accept': response(r,dict(disposition='started'))
    elif kind == 'clear_queue':
        if mode != 'stubborn': response(r,dict(steering=[],followUp=[]))
    elif kind == 'abort':
        if mode == 'stubborn': time.sleep(20)
        if mode == 'late_cancel': final('Late result must be fenced.');settle()
        response(r)
# Explicit stdin close is the real documented shutdown request.
(root/'closed').touch()
