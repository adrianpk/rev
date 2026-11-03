#!/usr/bin/python3
# Controlled RPC peer. It never performs a provider request or reads credentials.
import json, os, sys, time
from pathlib import Path
root = Path(__file__).parent
mode = (root / 'mode').read_text()
if '--version' in sys.argv:
    (root / 'version-pid').write_text(str(os.getpid()))
    if mode == 'version_hang': time.sleep(20)
    if mode == 'version_gate':
        while not (root/'release-version').exists(): time.sleep(.005)
    print('9.0.0' if mode == 'version' else '1.3.0', flush=True)
    sys.exit(0)
(root / 'pid').write_text(str(os.getpid()))
(root / 'args').write_text(json.dumps(sys.argv[1:]))
(root / 'cwd').write_text(os.getcwd())
(root / 'rpc-started').touch()
if '--extension' in sys.argv:
    resource = Path(sys.argv[sys.argv.index('--extension')+1])
    assert resource.stat().st_size <= 65536
    (root/'resource-copy').write_bytes(resource.read_bytes())
    (root/'resource-mode').write_text(json.dumps(dict(file=resource.stat().st_mode & 0o777, parent=resource.parent.stat().st_mode & 0o777, link=resource.is_symlink())))
session = Path(sys.argv[sys.argv.index('--session')+1]) if '--session' in sys.argv else None
count = 0
model = sys.argv[sys.argv.index('--model')+1] if '--model' in sys.argv else (root/'resolved-model').read_text() if (root/'resolved-model').exists() else 'fixture/controlled'
provider, model_id = model.split('/',1)
if session:
    import resource
    (root/'file-limit').write_text(json.dumps(resource.getrlimit(resource.RLIMIT_FSIZE)))
def emit(v):
    sys.stdout.write(json.dumps(v, ensure_ascii=False) + '\n')
    sys.stdout.flush()
def response(r, data=None, success=True):
    emit(dict(type='response', id=r['id'], command=r['type'], success=success, data=data))
def final(text=None):
    if text is None:
        text = (root/'result').read_bytes().decode('utf-8') if (root/'result').exists() else 'A selection explanation.\u2028With Unicode.'
    emit(dict(type='message_end', message=dict(role='assistant', stopReason='stop', content=[dict(type='text',text=json.dumps(dict(version=1,text=text),ensure_ascii=False))])))
def settle(): emit(dict(type='agent_settled',aborted=False))
for line in sys.stdin:
    r = json.loads(line)
    with (root/'requests').open('a') as log: log.write(json.dumps(r)+'\n')
    kind = r['type']
    if kind == 'get_state':
        response(r, dict(isStreaming=False, isCompacting=False, messageCount=1 if mode=='history' else count, pendingMessageCount=0,model=dict(provider=provider,id=model_id),**(dict(sessionFile=str(session)) if session else {})))
        if mode == 'stdin_stall': time.sleep(20)
    elif kind == 'get_available_models':
        response(r,dict(models=[dict(provider='fixture',id='controlled'),dict(provider='another',id='model')]))
    elif kind == 'switch_session':
        session=Path(r['sessionPath'])
        count=len(session.read_text().splitlines())
        response(r,dict(cancelled=False))
    elif kind == 'prompt' and r['message'].startswith('/rev-discovery '):
        grant = json.loads(r['message'].split(' ',1)[1])
        (root/'grant').write_text(json.dumps(grant))
        tools = ['rev_read','rev_list','rev_search']
        if mode == 'admit_extra': tools.append('bash')
        if mode == 'admit_missing': tools.pop()
        observed = Path(grant['root']).stat()
        same_root = str(observed.st_dev) == grant['dev'] and str(observed.st_ino) == grant['ino']
        notice = dict(type='extension_ui_request',id='admit-notice',method='notify',message=json.dumps(dict(type='rev-discovery-v1',nonce=grant['nonce'],tools=tools,admitted=mode!='admit_rejected' and same_root)))
        if mode == 'admit_partial': response(r,dict(disposition='handled'));continue
        emit(notice)
        response(r,dict(disposition='handled'))
    elif kind == 'prompt':
        (root/'prompt').write_text(r['message'])
        count+=1
        if session:
            os.umask(0o077)
            with session.open('a') as history: history.write(json.dumps(dict(fixture=True,message=r['message']))+'\n')
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
        if mode.startswith('directory_'):
            emit(dict(type='turn_start'))
            emit(dict(type='message_end',message=dict(role='assistant',stopReason='toolUse',content=[dict(type='toolCall',id='read-0',name='rev_read',arguments=dict(path='source.txt'))])))
            count = 33 if mode == 'directory_calls' else 5 if mode == 'directory_bytes' else 2 if mode == 'directory_repeated' else 1
            for n in range(count):
                tool = 'bash' if mode == 'directory_extra' else 'rev_read'
                identity = 'read-0' if mode == 'directory_repeated' else 'read-'+str(n)
                emit(dict(type='tool_execution_start',toolName=tool,toolCallId=identity))
                if mode == 'directory_unfinished': continue
                if mode == 'directory_update': emit(dict(type='tool_execution_update',toolName=tool,toolCallId=identity,partialResult={}))
                content = [dict(type='image',data='x')] if mode == 'directory_content' else [dict(type='text',text='x'*(1000000 if mode == 'directory_bytes' else 10))]
                emit(dict(type='tool_execution_end',toolName=tool,toolCallId='other' if mode == 'directory_unmatched' else identity,result=dict(content=content,details=dict(fatal=mode=='directory_fatal')),isError=False))
            emit(dict(type='turn_start'))
        if mode == 'stderr':
            sys.stderr.write('diagnostic-only\n' * 30000);sys.stderr.flush()
        if mode == 'malformed':
            sys.stdout.write('{invalid}\n');sys.stdout.flush();continue
        if mode == 'no_lf':
            sys.stdout.write('x' * (2*1024*1024+1));sys.stdout.flush();continue
        if mode == 'ui':
            emit(dict(type='extension_ui_request',id='dialog',method='input',title='SECRET-FIXTURE-MUST-NOT-LEAK'));continue
        if mode == 'tool':
            emit(dict(type='tool_execution_start',toolName='bash'));continue
        if mode == 'compaction':
            emit(dict(type='compaction_start',reason='threshold'))
            emit(dict(type='compaction_end',reason='threshold',aborted=False,willRetry=False,result=dict(summary='fixture history')))
        if mode == 'compaction_turns':
            for n in range(9):
                if n == 4:
                    emit(dict(type='compaction_start',reason='threshold'))
                    emit(dict(type='compaction_end',reason='threshold',aborted=False,willRetry=False,result=dict(summary='fixture history')))
                emit(dict(type='turn_start'))
            continue
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
