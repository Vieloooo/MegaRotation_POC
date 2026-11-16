#!/usr/bin/env python3
"""Read-only Git/source/evidence audit for M0; outputs only in /tmp/m0-verification."""
import hashlib,json,subprocess
from pathlib import Path
R=Path('/home/jy/.codex/worktrees/440b/megaroation_poc')
O=Path('/tmp/m0-verification')
D=Path('/mnt/reth/jy/megaroation_poc/.build/commander-jobs')
C='a0073f336357aa840467cf623ba98334416787ee'
B='ada48a584ad4de15e302c479ee8cd41b9e2a8d9b'
P='d5cbecb24836af80f8cd5414052043a17ca2ae17'
F='fec7ade1675cb60dd16a16fb28ebe81e3f76f508'
checks=[]; artifacts=[]; sha_cache={}
def git(*args):return subprocess.check_output(['git',*args],cwd=R).decode().strip()
def blob(ref,path):return subprocess.check_output(['git','show',f'{ref}:{path}'],cwd=R)
def sha(raw):return hashlib.sha256(raw).hexdigest()
def check(name,got,want=True):
 checks.append({'check':name,'observed':got,'expected':want,'pass':got==want});return got==want
def fsha(path):
 p=Path(path);key=str(p.resolve())
 if key not in sha_cache:
  h=hashlib.sha256()
  with p.open('rb')as f:
   while chunk:=f.read(1024*1024):h.update(chunk)
  sha_cache[key]=h.hexdigest()
 return sha_cache[key]
def ah(path,expected,source):
 p=Path(path)
 actual=fsha(p) if p.is_file() else None
 artifacts.append({'path':str(p),'expected_sha256':expected,'actual_sha256':actual,'source':source,'pass':actual==expected})
 return actual==expected
def gsha(path,expected,source):
 actual=sha(blob(C,path));checks.append({'check':'Git input '+path,'source':source,'expected':expected,'observed':actual,'pass':actual==expected})
def load(path):return json.loads(Path(path).read_text())
def js(ref,path):return json.loads(blob(ref,path))
def diff(a,b,paths):return git('diff','--name-status',a,b,'--',*paths).splitlines()
def tree(ref,path):
 r=subprocess.run(['git','rev-parse',f'{ref}:{path}'],cwd=R,capture_output=True,text=True)
 return r.stdout.strip() if r.returncode==0 else None
def ancestor(a,b):return subprocess.run(['git','merge-base','--is-ancestor',a,b],cwd=R).returncode==0
def tree_records(ref,paths):
 raw=subprocess.check_output(['git','ls-tree','-r','-z',ref,'--',*paths],cwd=R)
 rows=[]
 for line in raw.split(b'\0'):
  if not line:continue
  spec,path=line.split(b'\t',1);mode,kind,oid=spec.decode().split();path=path.decode()
  rows.append({'path':path,'mode':mode,'kind':kind,'git_oid':oid,'sha256':sha(blob(ref,path)) if kind=='blob' else None})
 return rows
components={
'03':dict(delivery='e9c15e04bc83b335668f3ecd837367aa87344a89',tested='e9c15e04bc83b335668f3ecd837367aa87344a89',source=['poc/crates/types','poc/crates/authorization','poc/crates/core','poc/Cargo.toml','poc/Cargo.lock','poc/proto'],dependencies=['poc/vendor','poc/crates/sim-runtime','poc/crates/sim-hub','poc/crates/replica-worker'],harness=['poc/verification/mock-plan-execution']),
'02':dict(delivery='e1a435712ec492cae2bbff120a51ffd507eebf45',tested='4fd6f115bd969b0a8f252d7933edd21c27f00a8f',source=['poc/crates/sim-transport'],dependencies=[],harness=['poc/verification/sim-communication']),
'06':dict(delivery='4b034dfbb6c3e1c7143208347b546daab8dfe298',tested='8ba78f1ace3b383caa1c4360401c645d25907326',source=['poc/crates/publisher','poc/crates/sim-espresso'],dependencies=[],harness=['poc/verification/publisher-espresso']),
'07':dict(delivery='788291735ce6db3c3eb78b4594a28d730cce71cd',tested='8907ad64d882673d6ef2142defdf6bff2e208a71',source=['poc/crates/viewer-gfr'],dependencies=['poc/crates/types','poc/crates/authorization','poc/crates/core','poc/crates/sim-espresso','poc/Cargo.toml','poc/Cargo.lock','poc/proto','poc/vendor'],harness=['poc/verification/viewer-gfr','poc/verification/mock-plan-execution/implementation-v1']),
'16':dict(delivery='5c34b400ba5245d7775a281566e21bf59708b147',tested='6248072b3fd0eee1ae468f0961a4d3589291cba6',source=['poc/crates/espresso-compatibility','poc/scenarios/espresso-compatibility'],dependencies=['poc/crates/sim-espresso'],harness=['poc/verification/espresso-compatibility']),
'09':dict(delivery='c443ae925ed549b046731c2d8ef3dd9ce0311d24',tested='87daee8e9e77bbe27c337383d873a5eaf4a694ba',source=['poc/contracts','poc/crates/l1-adapter'],dependencies=[],harness=['poc/verification/anvil-l1'])}
for name,x in components.items():
 check(name+' delivery is ancestor',ancestor(x['delivery'],C))
 check(name+' tested is delivery ancestor',ancestor(x['tested'],x['delivery']))
 paths=x['source']+x['dependencies']
 x['tested_to_delivery_source_diff']=diff(x['tested'],x['delivery'],paths)
 x['delivery_to_candidate_source_diff']=diff(x['delivery'],C,paths)
 x['tested_to_delivery_harness_diff']=diff(x['tested'],x['delivery'],x['harness'])
 x['delivery_to_candidate_harness_diff']=diff(x['delivery'],C,x['harness'])
 for k in ['tested_to_delivery_source_diff','delivery_to_candidate_source_diff','delivery_to_candidate_harness_diff']:check(name+' '+k,x[k],[])
 x['path_trees']=[{'path':p,'tested_tree_or_blob':tree(x['tested'],p),'delivery_tree_or_blob':tree(x['delivery'],p),'candidate_tree_or_blob':tree(C,p)}for p in paths+x['harness']]
 x['candidate_file_hashes']=tree_records(C,paths+x['harness'])
 x['note']='Post-test verification packaging is checked separately against historical raw evidence hashes; it is not claimed to have been committed at the tested SHA.' if x['tested_to_delivery_harness_diff'] else 'Source and harness trees match exactly through delivery and candidate.'
check('02 production tree',tree(C,'poc/crates/sim-transport'),'3d2ff54ca9231d2611e424599b0db73170faa9a2')
check('07 contains 03',ancestor(components['03']['delivery'],components['07']['delivery']))
check('07 contains 06',ancestor(components['06']['delivery'],components['07']['delivery']))
check('16 contains 06',ancestor(components['06']['delivery'],components['16']['delivery']))
check('Plan input ancestor',ancestor(P,C))
check('Primary plan ancestor',ancestor('4a9a6b0a92a11e5778833988bb6524d929f4608e',C))
check('Plan upgrade only state',diff('4a9a6b0a92a11e5778833988bb6524d929f4608e',P,['.']),['M\tpoc/docs/commander/state.json'])
check('Commander exact authorized plan',diff(P,C,['poc/docs/commander']),[])
planfiles=git('ls-tree','-r','--name-only',P,'--','poc/docs/milestones').splitlines()
for p in planfiles:check('Plan file '+p,tree(P,p),tree(C,p))
check('Milestones additions only own contract',diff(P,C,['poc/docs/milestones']),['A\tpoc/docs/milestones/results/M0-task-contract-v1.md'])
check('Baseline prototype/proto/vendor unchanged',diff(B,C,['poc/prototypes','poc/proto','poc/vendor']),[])
check('BFT pin',tree(C,'regional_BFT_sim'),'691cfc14daaa53c1a08cbd87dd3fe0cd60a214e6')
prototype_fresh_diff=diff(F,C,['poc/prototypes/i1b-runtime'])
check('Historical Fresh differs',bool(prototype_fresh_diff))
for p in ['poc/crates/batch-services','poc/crates/derivation']:
 check('Excluded component absent '+p,tree(C,p),None)
for ref in ['codex/poc-native-recovery','codex/poc-native-recovery-resume','codex/poc-batch-services','codex/poc-derivation']:
 check('Forbidden branch HEAD not ancestor '+ref,ancestor(git('rev-parse',ref),C),False)
newmerges=git('rev-list','--first-parent',f'{B}..{C}').splitlines()
check('Eight first-parent integration commits',len(newmerges),8)
check('main remains original',git('rev-parse','main'),B)
check('No main checked out','branch refs/heads/main' in git('worktree','list','--porcelain'),False)

# Immutable historical raw artifacts, preserving their original execution identity.
for rel,expected,typ in [
('03/artifacts/verification/full03/'+components['03']['tested']+'/run-manifest.json','a25ee38b4ea5eefd36e4302eedca04e5cc282f00008e537a7803b98ccc8c4da7','files'),
('03/artifacts/simulation/'+components['03']['tested']+'/run-manifest.json','47da8a35da3f22b4295916865c6d30b1f66db5be05ed9c625a71c47f072ebfa1','artifacts')]:
 p=D/rel;ah(p,expected,'03 commander acceptance');j=load(p);check('03 exact manifest candidate '+typ,j['candidate_sha'],components['03']['tested'])
 for k,v in j[typ].items():
  ah(p.parent/k,v,'03 '+typ) if isinstance(v,str) else ah(v['path'],v['sha256'],'03 '+typ)
 for k in ['expected_vector_sha256','additional_input_sha256']:
  for path,digest in j.get(k,{}).items():gsha('poc/verification/mock-plan-execution/implementation-v1/'+path,digest,'03 '+k)
# checksum-list reader retains both the list hash and every declared entry.
def sums(path,expected=None,source='checksum list'):
 p=Path(path)
 if expected:ah(p,expected,source+' list')
 else:artifacts.append({'path':str(p),'actual_sha256':fsha(p),'source':source+' unanchored list (entry hashes remain checked)','pass':True})
 n=0
 for line in p.read_text().splitlines():
  if not line.strip():continue
  digest,filename=line.split(maxsplit=1);filename=filename.lstrip('*')
  q=Path(filename);q=q if q.is_absolute() else p.parent/q
  ah(q,digest,source);n+=1
 return n
p=D/'02/artifacts/verification/sim-communication/kernel-v1/4fd6f11/final-verifier.json'
ah(p,'697e699a9a0f40a2958016f14c167aeb42adb056e9138eec8484a91650a3572a','02 delivery report');j=load(p)
for key in ['artifact_sha256','harness_sha256']:
 for path,digest in j[key].items():
  if '/poc/verification/' in path:gsha(path[path.index('poc/verification/'):],digest,'02 '+key)
  elif path.startswith('poc/'):gsha(path,digest,'02 '+key)
  else:ah(path,digest,'02 '+key)
p=D/'02/artifacts/simulation/sim-communication/kernel-v1/4fd6f11'
ah(p/'manifest.json','64a1ecdb4a496a7820cfaf0317f3a6ed43e74ed491b7d4dbf32bd09ad6b288f4','02 simulation report')
sums(p/'SHA256SUMS','48e19904a7888386c5835546c03e46a6122ec2c3733171dccba1774a4074e50a','02 matrix')
p=D/'06/artifacts/verification/8ba78f1ace3b383caa1c4360401c645d25907326/independent-run1'
sums(p/'SHA256SUMS','881b45c9920eed281b0d83a730a690b6b30328bd1e5df65b61b8706df5dc6df0','06 independent actual-file recovery')
for f,digest in {'tests/independent.rs':'c9b20f972051e09e475caf39d50fc72af190d9c685a1b1f5fb492c3d7ea9d4f9','Cargo.toml':'35f63f4c95ac3c39d74570944fbcc87f0b78eb26f9117cb11e3bb09fa2ef4939','Cargo.lock':'870e83af3e2133e3a06e570b05782a00e8ef0823f923dccd626680c1f00ddb4f'}.items():gsha('poc/verification/publisher-espresso/independent-harness/'+f,digest,'06 independent candidate review')
j=js(C,'poc/verification/publisher-espresso/simulation-results.json');p=Path(j['evidence']['absolute_path'])
gsha('poc/verification/publisher-espresso/run-matrix.py',j['matrix_runner_sha256'],'06 historical matrix runner')
sums(p/'SHA256SUMS',j['evidence']['manifest_sha256'],'06 full matrix')
j=js(C,'poc/verification/viewer-gfr/integrated-result-8907ad6.json');p=Path(j['evidence_directory'])
for f,digest in j['raw_files'].items():ah(p/f,digest,'07 integrated independent report')
j=js(C,'poc/verification/viewer-gfr/m2-result-de628c2.json')
# All source/harness bytes embedded in the prior independent result are handled below generically.
def hunt_source_hashes(obj,source):
 if isinstance(obj,dict):
  for k,v in obj.items():
   if isinstance(v,str) and len(v)==64 and k.startswith('poc/'):gsha(k,v,source)
   elif isinstance(v,(dict,list)):hunt_source_hashes(v,source)
 elif isinstance(obj,list):
  for v in obj:hunt_source_hashes(v,source)
hunt_source_hashes(j,'07 m2 source hashes')
p=D/'07/artifacts/simulation/gfr-8907ad6-g1-g2-v1'
ah(p/'capture.json','22ffb73876e98efc6327ff8adcea648e1734f2e9a874bc7e57e1c783752fcf6c','07 simulation report')
j=load(p/'capture.json')
for f,digest in j['source_sha256_after'].items():gsha(f,digest,'07 capture input hash')
for f in p.glob('*.run.json'):
 j=load(f)
 for ext,key in [('jsonl','stdout_sha256'),('stderr','stderr_sha256')]:ah(p/(f.name.removesuffix('.run.json')+'.'+ext),j[key],'07 actual replay '+f.name)
j=js(C,'poc/verification/viewer-gfr/simulation/summary.json')
for f,digest in j['tester_script_sha256'].items():gsha('poc/verification/viewer-gfr/simulation/'+f,digest,'07 historical simulation checker/runner')
for f,digest in j['input_hashes_checked_after_capture'].items():gsha(f,digest,'07 supplemental captured input')
# The 16 seal enumerates both successful/failed official runs and final independent helpers/matrix.
p=D/'16/artifacts/lead-seal-6248072/checksums-audit.json'
ah(p,'2bdc1e1f3bcb96cb17895beb45edece562f85e52bbb6163664ab51b09bda4a01','16 delivery seal')
for row in load(p)['checks']:check('16 checksum entry count '+row['manifest'],sums(row['manifest'],row['sha256'],'16 preserved evidence'),row['entries'])
for f,digest in {'check_capture.py':'e42e68df3f40c991dcce199ba6fa53d03fd2c6d75f243d0bf3ace9c25ead157a','run_offline_checks.py':'1a949eb5f9c8eb2b0025fab9b8aacd87598a4432ecd76756a8aa493c38234f34','independent-harness/tests/independent.rs':'3e7abd76ac8fec65585f0a98dda3a93b6bbd01bd917783bd5b126444645f4c9f','independent-harness/Cargo.lock':'f16db4763b12615cd1764b88b8010c8ac8d619eff9a0813cac47a045bab7ca8b'}.items():gsha('poc/verification/espresso-compatibility/'+f,digest,'16 final helper-version review')
j=js(C,'poc/verification/anvil-l1/reports/preserved-artifacts.json')
for row in j['entries']:gsha(row['preserved'],row['sha256'],'09 original preserved result')
j=js(C,'poc/verification/anvil-l1/reports/simulation-87daee8-manifest.json')
for f,digest in j['artifactSha256'].items():ah(f,digest,'09 full Tester matrix')
sums(D/'09/artifacts/anvil-l1-verifier-87daee8-run2/evidence-sha256.txt',None,'09 independent raw receipts')
# Candidate checkout matches committed relevant inputs, even if another owner adds unrelated M0 runner files.
allfiles={x['path']:x for c in components.values()for x in c['candidate_file_hashes']}
for p,x in allfiles.items():
 if x['kind']=='blob':check('Working file matches candidate '+p,fsha(R/p),x['sha256'])
check('Head remains named source candidate',git('rev-parse','HEAD'),C)
failed=[c for c in checks if not c['pass']];bad=[a for a in artifacts if not a['pass']]
result={'schema':'M0-SOURCE-TREE-BINDING-1','candidate':C,'base_main':B,'plan':P,'primary_plan':'4a9a6b0a92a11e5778833988bb6524d929f4608e','worktree':str(R),'components':components,'baseline_prototype_tree':tree(B,'poc/prototypes/i1b-runtime'),'candidate_prototype_tree':tree(C,'poc/prototypes/i1b-runtime'),'historical_fresh_tree':tree(F,'poc/prototypes/i1b-runtime'),'historical_fresh_source_diff':prototype_fresh_diff,'first_parent_commits':newmerges,'checks':checks,'historical_artifact_checks':artifacts,'counts':{'identity_and_input_checks':len(checks),'historical_artifact_checks':len(artifacts),'unique_disk_files_hashed':len(sha_cache),'failed_identity_checks':len(failed),'failed_artifact_checks':len(bad)},'verdict':'PASS_SOURCE_AND_EVIDENCE_BINDING' if not failed and not bad else 'FAIL','scope':'Read-only identity and integrity audit, no new runtime tests or cryptographic validation; original evidence keeps original execution SHA. M0 Tester gates still pending.','git_status_after':git('status','--short')}
(O/'source-tree-binding.json').write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
print(json.dumps({'verdict':result['verdict'],'counts':result['counts'],'failed':failed,'bad_artifacts':bad},indent=2))
