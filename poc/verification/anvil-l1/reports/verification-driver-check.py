import json,hashlib
from pathlib import Path
root=Path('/home/jy/.codex/worktrees/65ef/megaroation_poc/.build/artifacts/anvil-l1-verifier-87daee8-run2')
allowed={
 'tick':set(), 'submit':{'eth_sendRawTransaction','eth_getTransactionByHash'},
 'mine':{'eth_blockNumber','evm_setNextBlockTimestamp','evm_mine','eth_getBlockByNumber','eth_getTransactionReceipt'},
 'finalize':set(), 'deliver':set(), 'read':{'eth_call'}}
reports=[]
for pause in (300,650):
    rows=json.loads((root/f'pause-{pause}/driver-trace.json').read_text())
    diagnostic=json.loads((root/f'pause-{pause}/diagnostic-rpc.json').read_text())
    seen={};actual=None;mines=0;lost=[];reconciled=[];reads=0;unchanged=[]
    for row in rows:
        if 'command' not in row:
            assert all(x['request']['method'] in {'eth_chainId','anvil_getAutomine','eth_blockNumber','eth_getBlockByNumber'} for x in row['startup_rpc'])
            continue
        cmd=row['command'];rep=row['reply'];rpc=row['rpc'];id=cmd['id'];op=cmd['op'];status=rep['status']
        methods=[x['request']['method'] for x in rpc]
        assert set(methods)<=allowed[op],(op,methods)
        mines+=methods.count('evm_mine')
        if id in seen:
            prior_cmd,prior_rep=seen[id]
            if prior_cmd!=cmd:
                assert status=='reject' and not rpc
                continue
            if not rpc:
                assert rep==prior_rep
                unchanged.append(id)
                continue
            assert op=='mine' and prior_rep['status']=='wait' and status=='ready'
            assert 'evm_mine' not in methods and 'evm_setNextBlockTimestamp' not in methods
            assert actual['included_height']==cmd['slot']-1
            reconciled.append(id)
        if actual:
            if status!='ready':
                assert all(rep[k]==actual[k] for k in ('reference_us','included_height','finalized_height','visible_heights'))
            else:
                if op!='tick':assert rep['reference_us']==actual['reference_us']
                if op!='mine':assert rep['included_height']==actual['included_height']
                if op!='finalize':assert rep['finalized_height']==actual['finalized_height']
                if op!='deliver':assert rep['visible_heights']==actual['visible_heights']
        if status=='ready':
            if op=='tick':assert rep['reference_us']==cmd['reference_us']
            if op=='mine':
                assert rep['included_height']==cmd['slot']
                assert rep['result']['included_block']['timestamp']==1700000000+2*cmd['slot']
                assert rep['reference_us']>=2_000_000*cmd['slot']
            if op=='finalize':
                assert rep['finalized_height']==cmd['through_height']<=rep['included_height']
                assert rep['reference_us']>=2_000_000*cmd['through_height']+2_000_000
            if op=='read':
                assert cmd['query']['height']<=rep['visible_heights'][cmd['reader']]
                reads+=1
            if op=='deliver':assert all(h<=rep['finalized_height'] for h in rep['visible_heights'].values())
        if op=='mine' and status=='wait' and 'evm_mine' in methods:
            hit=next(x for x in rpc if x['request']['method']=='evm_mine')
            assert 'Ok' in hit['result'] and hit['response']['result'] is not None
            assert rep['included_height']==cmd['slot']-1
            lost.append(id)
        actual=rep;seen[id]=(cmd,rep)
    assert len(lost)==len(reconciled)==1 and lost==reconciled and mines==7
    assert actual['reference_us']==16_000_000 and actual['included_height']==actual['finalized_height']==7
    assert actual['visible_heights']=={'A':7,'B':7}
    assert all(x['request']['method'] in {'eth_chainId','eth_blockNumber','eth_getTransactionReceipt','eth_call'} for x in diagnostic)
    reports.append({'pauseMs':pause,'status':'PASS','commands':sum('command' in r for r in rows),'mineRpcCount':mines,'lostCompletedMineCommand':lost,'reconciledWithoutMutation':reconciled,'cachedReplayNoRpc':unchanged,'readyReaderQueries':reads,'finalReferenceUs':actual['reference_us'],'finalPrefix':7})
assert (root/'pause-300/virtual-trace.json').read_bytes()==(root/'pause-650/virtual-trace.json').read_bytes()
print(json.dumps({'scope':'independent command/RPC separation and actual lost-mine-response replay','runs':reports,'status':'PASS'},indent=2))
