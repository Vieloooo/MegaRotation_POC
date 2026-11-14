import fs from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { Presentation, PresentationFile } from '@oai/artifact-tool';

const ROOT='/Users/jin/projects/megaeth_sequenser_rotation';
const WORK=path.join(ROOT,'presentations/2026-09-28-design-tradeoffs');
const BUILD=path.join(WORK,'.build');
const SKILL='/Users/jin/.codex/plugins/cache/openai-primary-runtime/presentations/26.904.11930/skills/presentations';
const PYTHON='/Users/jin/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3';
const VERSION=process.env.DECK_VERSION || 'v4';
process.env.RUNTIME_NODE_MODULES='/Users/jin/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules';
process.env.RUNTIME_NODE='/Users/jin/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node';
process.env.RUNTIME_PYTHON=PYTHON;
process.env.RUNTIME_BIN_DIR='/Users/jin/.cache/codex-runtimes/codex-primary-runtime/dependencies/bin/override';
const FONT='Arial Unicode MS';
const fontPolicy={basis:'design',families:[FONT]};
const {finalizePresentation}=await import(pathToFileURL(path.join(SKILL,'container_tools/artifact_tool_utils.mjs')).href);
const p=Presentation.create({slideSize:{width:1280,height:720}});
const C={ink:'#172C3B',muted:'#586B78',blue:'#246A8E',light:'#EAF2F6',teal:'#167D70',tealLight:'#E8F4F0',amber:'#AA6E13',amberLight:'#FFF2DB',red:'#B25145',redLight:'#FAEDE8',line:'#C6D3DA',gray:'#F2F5F6',white:'#FFFFFF'};
const metadata=[];

function text(s,str,x,y,w,h,size=28,color=C.ink,bold=false,align='left'){
 const z=s.shapes.add({geometry:'textbox',name:`text-${s.id}-${s.shapes.items?.length||str.slice(0,15)}`,position:{left:x,top:y,width:w,height:h},fill:'none',line:{fill:'none',width:0}});
 z.text=str;
 z.text.style={typeface:FONT,fontSize:size,bold,color,alignment:align,verticalAlignment:'middle',autoFit:'none',wrap:'square',insets:{left:0,right:0,top:0,bottom:0}};
 return z;
}
function rect(s,x,y,w,h,fill=C.white,stroke=C.line,width=2,dashed=false){return s.shapes.add({geometry:'rect',position:{left:x,top:y,width:w,height:h},fill,line:{fill:stroke,width,style:dashed?'dashed':'solid'}});}
function node(s,label,x,y,w,h,{fill=C.light,stroke=C.blue,color=C.ink,size=26,bold=false,dashed=false}={}){
 const z=rect(s,x,y,w,h,fill,stroke,2,dashed);z.text=label;z.text.style={typeface:FONT,fontSize:size,bold,color,alignment:'center',verticalAlignment:'middle',autoFit:'none',wrap:'square',insets:{left:10,right:10,top:6,bottom:6}};return z;
}
function line(s,x1,y1,x2,y2,color=C.line,width=2,dashed=false){return s.shapes.add({geometry:'line',position:{left:x1,top:y1,width:x2-x1,height:y2-y1},fill:'none',line:{fill:color,width,style:dashed?'dashed':'solid'}});}
function arrow(s,a,b,{from='right',to='left',color=C.blue,dashed=false,kind='elbow',both=false}={}){const z=s.shapes.connect(a,b,{kind,fromSide:from,toSide:to,line:{fill:color,width:2.5,style:dashed?'dashed':'solid'},tail:{type:'triangle',width:'med',length:'med'},...(both?{head:{type:'triangle',width:'med',length:'med'}}:{})});z.bringToFront();return z;}
function region(s,name,x,y,w=200,h=150,{active=false,failed=false,sub='区域 BFT',leader=0}={}){
 const g=rect(s,x,y,w,h,failed?C.redLight:active?C.light:C.gray,failed?C.red:active?C.blue:C.line,2);
 text(s,name,x+12,y+10,w-24,30,24,failed?C.red:C.ink,true);
 for(let k=0;k<3;k++){
  const z=s.shapes.add({geometry:'ellipse',position:{left:x+20+k*(w-65)/2,top:y+52,width:34,height:34},fill:failed?C.redLight:(k===leader&&active?C.blue:C.white),line:{fill:failed?C.red:C.blue,width:1.6}});
  z.text=`S${k+1}`;z.text.style={typeface:FONT,fontSize:15,color:k===leader&&active?C.white:C.blue,alignment:'center',verticalAlignment:'middle',insets:{left:0,right:0,top:0,bottom:0}};
 }
 text(s,sub,x+10,y+98,w-20,h-104,21,C.muted,false,'center');
 return g;
}
function notes(s,title,minutes,body,sources){
 const content=`建议用时：${minutes} 分钟\n\n${body}\n\n资料来源（本地设计记录）：\n${sources.map(q=>ROOT+'/'+q).join('\n')}`;
 s.speakerNotes.textFrame.setText(content);metadata.push({slide:metadata.length+1,title,minutes,notes:body,sources});
}
function base(title){const s=p.slides.add();s.background.fill=C.white;text(s,title,64,42,1144,65,43,C.ink,true);line(s,64,128,1216,128);text(s,String(p.slides.items.length).padStart(2,'0'),1140,668,76,30,18,C.muted,false,'right');return s;}
function takeaway(s,str,color=C.ink){text(s,str,64,600,1140,54,28,color,true);}
function side(s,gain,cost,{x=905,y=225,w=300}={}){text(s,'收益',x,y,w,42,27,C.teal,true);text(s,gain,x,y+46,w,100,26);text(s,'代价',x,y+180,w,42,27,C.amber,true);text(s,cost,x,y+226,w,130,26);}

// 1: A quiet cover, with no decorative source imagery.
{
const s=p.slides.add();s.background.fill=C.white;
text(s,'Sequencer streaming',72,165,1120,95,68,C.ink,true);
text(s,'架构与设计取舍',76,279,1100,78,52,C.blue,true);
text(s,'理想服务接口、运行场景与四条设计路线',78,416,1050,48,29,C.muted);
text(s,'2026 年 9 月 22–28 日设计回顾',78,602,900,32,22,C.muted);
notes(s,'Sequencer streaming：架构与设计取舍',0.5,'这次只讨论整体服务与架构选择。核心问题是怎样在可验证的授权和前缀承诺下，以低延迟持续向 proposer 提供流，并以合理成本完成轮换。协议中的投票、锁、阈值和算法会另行讲解。',['research-index/2026-09-28-weekly-design/tradeoffs-zh.md']);
}
// 2: Ideal interface.
{
const s=base('理想接口：可验证、可接续的排序流');
const l1=node(s,'L1 授权与排班',396,166,330,62,{fill:C.gray,stroke:C.ink});
const users=node(s,'用户交易',65,355,157,85,{fill:C.white,stroke:C.line});
const service=rect(s,306,280,510,235,C.light,C.blue,2.5);
text(s,'Sequencer streaming service',326,301,470,44,29,C.blue,true,'center');
let prev;for(let i=0;i<4;i++){const b=node(s,`M${i+1}`,351+i*106,365,80,58,{fill:C.white,stroke:C.blue,size:24});if(prev)arrow(s,prev,b);prev=b;}
text(s,'有序数据 ＋ 授权／确认依据',330,451,462,39,25,C.ink,false,'center');
const prop=node(s,'Proposer P',928,355,272,85,{fill:C.gray,stroke:C.ink,size:28,bold:true});
const pub=node(s,'执行、封装与发布',928,529,272,72,{fill:C.white,stroke:C.line,size:25});
arrow(s,l1,service,{from:'bottom',to:'top',color:C.ink});arrow(s,users,service);arrow(s,service,prop);arrow(s,prop,pub,{from:'bottom',to:'top',color:C.muted});
text(s,'持续交付',819,303,110,31,21,C.blue,false,'center');
text(s,'换 leader / region 后，消费者仍沿同一条已确认历史继续',65,616,1110,38,27,C.teal,true);
notes(s,'理想接口',2,'先把 sequencing 视作独立服务。输入是交易和已生效的 L1 授权，输出是有序 mini-block 数据以及接收方可以独立检查的来源、连续性和确认依据。proposer 消费该流后负责执行、封装、DA/L1 发布。Sequencer 可为自己的排序目的执行，但该服务不强制其承担执行证明。架构图中的节点数量只示意角色，不表示委员会大小或容错阈值。图中的 M 是排序服务的抽象输出，不声称完全等同于当前原生 MegaETH header。跨 region 仍不可回滚是理想接口目标，不是所有历史方案均已提供的保证。',['meetup_record/meeting_0922_requirements.md','formalization/outline.md','research-index/2026-09-28-weekly-design/tradeoffs-zh.md']);
}
// 3: Requirements and priorities, flat typography.
{
const s=base('需求：明确承诺，优先优化常态性能');
text(s,'服务必须回答',66,169,520,42,29,C.ink,true);
text(s,'谁有权生产这段 stream？',66,235,510,43,30);
text(s,'P 能拿到数据并验证来源吗？',66,307,510,43,30);
text(s,'哪些前缀在轮换后仍要保留？',66,379,510,43,30);
text(s,'故障后何时恢复被接受的出流？',66,451,540,43,30);
line(s,646,175,646,540,C.line);
text(s,'性能优先级',704,169,500,42,29,C.blue,true);
text(s,'正常出流与日常交班',704,235,500,45,33,C.blue,true);
text(s,'低确认延迟，低通信与轮换开销',704,291,500,68,27);
text(s,'罕见整区事故可承担更长等待',704,391,500,72,28,C.amber);
text(s,'逐交易抗审查与精细激励后续讨论',704,500,500,62,24,C.muted);
takeaway(s,'Weak liveness：可接受的 stream 持续增长',C.teal);
notes(s,'需求与优先级',1.5,'9 月 24 日的明确偏好是 latency、communication 与 rotation cost 优先，其次 weak liveness，再其次 censorship。后来进一步明确外部 P 的不可回滚确认目标。安全承诺应先说清楚：预览可变，已承诺不可回滚的前缀必须继承。这里的 weak liveness 关注流增长，不保证每个用户交易在固定时间内进入流。故障恢复与惩罚归责也应分别处理。',['meetup_record/meeting_0922_requirements.md','research-index/2026-09-28-weekly-design/chat-index.md']);
}
// 4: Four architectural cases.
{
const s=base('四类运行场景，四种性能问题');
line(s,642,156,642,587);line(s,64,376,1216,376);
text(s,'01  正常区域内出流',66,158,520,37,27,C.blue,true);
const a=region(s,'Region A',76,217,215,122,{active:true,sub:'持续生产'});
const pa=node(s,'P',442,250,90,56,{fill:C.gray,stroke:C.ink});arrow(s,a,pa);
text(s,'确认延迟、pipeline interval、每批通信',76,343,530,25,22,C.muted);
text(s,'02  区域内换 leader',684,158,510,37,27,C.blue,true);
const boundary=rect(s,695,217,311,122,C.light,C.blue);
text(s,'同一 Region A',711,226,278,28,22,C.ink,true);
const aa=node(s,'S₁',722,274,75,47,{fill:C.white,size:24});const ab=node(s,'S₂',899,274,75,47,{fill:C.blue,color:C.white,size:24});arrow(s,aa,ab);
const pb=node(s,'P',1102,249,88,56,{fill:C.gray,stroke:C.ink});arrow(s,boundary,pb);
text(s,'换人停顿、前缀继承、恢复首批',695,343,510,25,22,C.muted);
text(s,'03  正常跨区交班',66,393,530,37,27,C.teal,true);
const ra=node(s,'Region A',78,466,170,70,{fill:C.light});const rb=node(s,'Region B',422,466,170,70,{fill:C.tealLight,stroke:C.teal});arrow(s,ra,rb,{color:C.teal});
text(s,'finalized anchor',252,431,169,29,21,C.teal,false,'center');
text(s,'日常边界停顿、状态传输、摊销成本',78,556,530,30,22,C.muted);
text(s,'04  整区失效恢复',684,393,510,37,27,C.amber,true);
const fa=node(s,'A 失效',696,466,125,70,{fill:C.redLight,stroke:C.red,color:C.red,size:24});
const co=node(s,'委员会 / L1',865,466,176,70,{fill:C.amberLight,stroke:C.amber,size:23});
const fb=node(s,'B 接续',1085,466,125,70,{fill:C.tealLight,stroke:C.teal,size:24});arrow(s,fa,co,{color:C.amber,dashed:true});arrow(s,co,fb,{color:C.amber});
text(s,'失效观察',822,435,123,24,18,C.amber);
text(s,'允许较长等待，仍须保留确认历史',696,556,510,30,22,C.muted);
takeaway(s,'优先减少高频成本与正常交班停顿');
notes(s,'四类运行场景',2,'左上是正常出流，右上是同一区域内换生产者。下方分别是常规地域交班与事故恢复。区域内换人需要区分计划任期结束和故障检测后接替。正常跨区在最新方案里由固定 batch quota 触发，下一地区先接收已 finalized 的锚点。右下委员会/L1 是最新候选架构的事故控制路径，图仅表示依赖关系，不表示故障区域必须在线授权退出。事故恢复还需要独立的已确认数据与安全截止条件。非活动区恢复只影响未来资格，不打断当前任期。',['research-index/2026-09-28-weekly-design/tradeoffs-zh.md','formalization/proposals/v3-status-entry-2026-09-28/outline-and-revision-plan-zh.md']);
}
// 5: Qualitative timeline, not a quantitative chart.
{
const s=base('确认延迟与连续出块间隔');
text(s,'同一批次等待确认时，后续批次可以继续推进',66,163,1110,48,30,C.muted);
const ys=[273,362,451];const starts=[285,430,575];const ends=[760,905,1050];
for(let i=0;i<3;i++){
 text(s,`Batch ${i+1}`,77,ys[i]-8,155,42,27,C.ink,true);
 const st=node(s,'提出',starts[i],ys[i],80,45,{fill:C.light,size:22});const en=node(s,'P 可验证确认',ends[i],ys[i],185,45,{fill:C.tealLight,stroke:C.teal,size:22});
 arrow(s,st,en,{color:C.blue});
}
text(s,'单批次 finality latency',410,225,480,38,27,C.blue,true);
text(s,'连续提案间隔',300,520,365,34,27,C.teal,true);
line(s,325,501,325,516,C.teal,2);line(s,470,501,470,516,C.teal,2);line(s,325,510,470,510,C.teal,2);
text(s,'示意图，无数值比例',890,551,302,29,20,C.muted,false,'right');
takeaway(s,'同时比较：确认时间、稳态间隔、消息／字节与 L1 证据成本');
notes(s,'确认延迟与连续出块间隔',1.5,'这张图只讲测量口径，没有按比例绘制协议时延。三批可同时处于确认流程，单批次的等待和连续出块间隔因此不同。P 的确认还要包括数据与证据到达。比较两条方案前必须固定同一确认层级，不能拿可撤回 QC 对比不可回滚 receipt。通信同时记消息数与字节数，另记跨区域数据与 L1 证明成本。Batch accumulation 与队列时间也不能藏在 proposal-ready 的计时起点之外。',['formalization/reviews/hotstuff-family-proposer-finality-2026-09-28.md','research-index/2026-09-28-weekly-design/tradeoffs-zh.md']);
}
// 6: Hierarchical coordination.
{
const s=base('方案 A：区域 BFT 与全局协调');
text(s,'GeoBFT 启发的分层结构',65,159,1000,37,27,C.muted);
const g=node(s,'全局协调与 checkpoint',226,224,516,77,{fill:C.gray,stroke:C.ink,size:29});
const a=region(s,'Region A',70,385,203,151,{active:true});const b=region(s,'Region B',330,385,203,151);const c=region(s,'Region C',590,385,203,151);
arrow(s,a,g,{from:'top',to:'bottom',kind:'elbow'});arrow(s,b,g,{from:'top',to:'bottom'});arrow(s,c,g,{from:'top',to:'bottom',kind:'elbow'});
side(s,'把快通信放在区域内\n形成明确的局部确认','内外层协调持续发生\n确认、存储与交接叠加');
takeaway(s,'局部更快，整体仍要承担跨区协调');
notes(s,'方案 A：区域 BFT 与全局协调',2,'最初从 GeoBFT 获得启发：将 Byzantine coordination 放在局部小集群，再进行跨区传播、共同采纳和轮换。本图画的是本项目的单 active region 适配，不是 GeoBFT 原论文逐项结构。历史具体实现包含区域 fast path/completion 和全局报告一致性加 FFG。需要承认每一层都承担真实的确认、数据保存和恢复义务。区域原语之后比较了同步 broadcast、PBFT 和 HotStuff family，这里只保留架构层。',['formalization/outline.md','formalization/reviews/consensus-sources.md','formalization/reviews/main-v1-cross-region-and-virtual-smr-2026-09-28.md']);
}
// 7: LMD head selection plus global finality.
{
const s=base('方案 B：区域 LMD-GHOST 与全局 FFG');
const a=region(s,'Active region',68,253,262,169,{active:true,sub:'LMD-GHOST 选 head'});
const preview=node(s,'可重组 preview',447,305,343,65,{fill:C.amberLight,stroke:C.amber,size:26});
const final=node(s,'全局持久确认',447,473,343,65,{fill:C.tealLight,stroke:C.teal,size:26});
const g=node(s,'全局报告一致性 + FFG', 68,465,262,80,{fill:C.gray,stroke:C.ink,size:23});
arrow(s,a,preview,{color:C.amber});arrow(s,a,g,{from:'bottom',to:'top',color:C.ink});arrow(s,g,final,{color:C.teal});
side(s,'区域可以先持续出流\n允许暂定尾部变化','强确认仍依赖全局层\n报告与数据成本仍在');
takeaway(s,'更早看到流，换来更晚的不可回滚承诺');
notes(s,'方案 B：区域 LMD-GHOST 与全局 FFG',2,'LMD-GHOST 负责选择本地暂定头，FFG 负责较慢的 checkpoint 最终确认。实际草稿还需要全局报告一致性先构造共同目标，因此 FFG 并没有消除全局 agreement。更低频的全局层可摊薄控制开销，同时延长最终确认等待，并留下更长的暂定尾部。该设计允许 preview 重组，不能拿 preview latency 与其他方案的永久确认直接比较。',['formalization/main.tex','formalization/reviews/lmd-ffg-redesign/README.md','formalization/reviews/ffg-composition-advisory.md']);
}
// 8: BFT regions as virtual CFT participants.
{
const s=base('方案 C：区域 BFT 封装成外层 CFT 节点');
text(s,'每个 region 模拟一个正确或停止的逻辑进程',66,157,1110,40,28,C.muted);
const outer=node(s,'外层 CFT',81,219,714,73,{fill:C.gray,stroke:C.ink,size:29});
const a=region(s,'Region A',76,374,211,160,{active:true,sub:'BFT SMR + wrapper'});const b=region(s,'Region B',330,374,211,160,{sub:'BFT SMR + wrapper'});const c=region(s,'Region C',584,374,211,160,{sub:'BFT SMR + wrapper'});
arrow(s,a,outer,{from:'top',to:'bottom',kind:'elbow'});arrow(s,b,outer,{from:'top',to:'bottom'});arrow(s,c,outer,{from:'top',to:'bottom',kind:'elbow'});
side(s,'跨区按逻辑区域协调\n可接入 L1 指定排班','复制整个外层进程行为\n输入、状态、输出与恢复', {y:215});
takeaway(s,'外层变轻的前提，是内层完成可靠的进程封装');
notes(s,'方案 C：区域 BFT 与外层 CFT',2,'需要复制的是整个外层进程的合法行为，不仅是业务交易流：输入认证、协议状态、计时器、持久承诺、外发消息、重传和恢复。一个恶意 regional representative 不能绕过内部状态独自对外发承诺。内层一次共识不能简单视为一条便宜物理消息。Steward 和 Nysiad 是过去检索到的相关先例，说明路线可行而非没有研究。L1 指定谁接班也不替代日志恢复与激活。我们后来选择先避免通用 wrapper 的成本。',['formalization/reviews/smr-protocol-wrapping-literature-2026-09-28.md','formalization/reviews/l1-scheduled-cft-rotation-2026-09-28.md']);
}
// 9: Current proposal with bounded committee scope.
{
const s=base('方案 D：区域 SMR、预设排班与状态委员会');
const l1=node(s,'L1：排班与状态记录',485,170,390,65,{fill:C.gray,stroke:C.ink,size:27});
const com=node(s,'状态委员会',72,285,210,66,{fill:C.amberLight,stroke:C.amber,size:25});
arrow(s,com,l1,{from:'top',to:'left',kind:'elbow',color:C.amber});
text(s,'低频状态更新',316,280,278,34,23,C.amber);
text(s,'已生效的授权依据',406,343,245,31,21,C.muted);
const a=region(s,'Region A',73,406,215,151,{active:true,sub:'区域 SMR'});const b=region(s,'Region B',573,406,215,151,{sub:'区域 SMR'});
arrow(s,a,b,{color:C.teal});
text(s,'正常满额交接',312,421,235,34,26,C.teal,true,'center');
text(s,'finalized anchor',314,504,231,32,22,C.teal,false,'center');
arrow(s,l1,b,{from:'bottom',to:'top',kind:'straight',color:C.ink,dashed:true});
side(s,'正常路径按既定规则走\n事故才承担额外协调','依赖状态记录与安全截止\n仍需保证数据与前缀继承',{y:210});
takeaway(s,'当前提案：减少常态协调，接受较慢的事故恢复',C.teal);
notes(s,'方案 D：L1 排班与状态委员会',2,'这是本周最后选定的架构方向。正常轮换在固定数量的应用 batches 完成后沿预定义列表选择下一 live region，旧 finalized anchor 是启动新区的前提。委员会维护业务 entry：batch height 与 region status map。只有当前 active region 被标为 unavailable 才触发强制轮换。非活动区恢复只改变资格。L1 虚线表示已生效的授权/状态依据，不表示正常每次交接都必须新增一笔 L1 交易。委员会仍须一致地认证 entries，并保证事故 cutoff 与已确认输出兼容。完整组合安全与恢复仍有实质工作。',['formalization/proposals/v3-status-entry-2026-09-28/outline-and-revision-plan-zh.md','formalization/proposals/v3-committee-rotation-2026-09-28/README.md']);
}
// 10: Settlement verification boundary.
{
const s=base('L1 验证：proposer 是否忠实使用源流');
const r=node(s,'区域 SMR\nstream + final receipt',68,195,292,105,{fill:C.light,size:25});
const q=node(s,'Proposer P',503,195,249,105,{fill:C.gray,stroke:C.ink,size:28,bold:true});
const l=node(s,'L1 / proof system',895,195,315,105,{fill:C.tealLight,stroke:C.teal,size:26});
arrow(s,r,q);arrow(s,q,l,{color:C.teal});
text(s,'核验内容与顺序、生产权限、确认依据、合法轮换',74,340,1120,48,30,C.ink,true);
line(s,64,420,1216,420);
text(s,'委员会认证 stream tip',70,454,520,42,29,C.amber,true);
text(s,'可能简化来源认证接口\n增加委员会对内容的权力与责任',70,511,510,84,27);
text(s,'委员会只维护状态与高度',693,454,517,42,29,C.teal,true);
text(s,'当前选择：缩小委员会职责\n内容仍由区域证据支持',693,511,517,84,27);
text(s,'执行正确性与 DA 可用性仍由独立机制保证',70,636,1110,30,23,C.muted);
notes(s,'L1 验证与委员会职责',1.5,'这是源共识域到 L1 的可验证输出接口，因此类似跨链验证问题。P 需要证明自己发布的数据忠实对应源 stream 及其 authority/rotation history。曾考虑让委员会进一步认证 stream tip，可能使来源认证更集中，但也增大了内容与 finality 的信任依赖。最新选择将委员会限制为状态与高度，内容由区域 final receipt 支持。高度本身不能证明不存在更高的已确认后缀，事故截止完整性仍需机制。Fraud proof 或 validity proof 是候选实现接口，当前没有 gas 或 prover 性能结论。执行正确性与 DA 可用性不由排序签名自动推出。',['meetup_record/meeting_0916_response.md','formalization/proposals/v3-status-entry-2026-09-28/outline-and-revision-plan-zh.md']);
}
// 11: Native editable comparison table.
{
const s=base('架构取舍：成本与信任依赖放在哪里');
const vals=[['路线','主要收益','主要代价'],['区域 BFT + 全局协调','局部确认清楚','两层协调与交接成本'],['区域 LMD + 全局 FFG','较早交付 preview','强确认等待全局层'],['区域 BFT + 外层 CFT','外层采用区域逻辑节点','通用进程封装成本'],['区域 SMR + 状态委员会','轻量常态路径','控制信任与事故恢复义务']];
const tab=s.tables.add({rows:5,columns:3,left:66,top:185,width:1144,height:368,columnWidths:[418,328,398],values:vals});
tab.borders.assign({fill:C.white,width:2,style:'solid'});
tab.cells.block({row:0,column:0,rowCount:5,columnCount:3}).assign({textStyle:{typeface:FONT,fontSize:27,color:C.ink},margins:{left:18,right:18,top:14,bottom:14},anchor:'center'});
for(let i=0;i<5;i++){tab.rows[i].height=i===0? sixty():77;for(let j=0;j<3;j++){const cell=tab.getCell(i,j);cell.fill=i===0?C.ink:(i===4?C.tealLight:i%2?C.gray:C.white);cell.text.style={typeface:FONT,fontSize:i===0?27:26,bold:i===0,color:i===0?C.white:C.ink};}}
takeaway(s,'同一比较基准：P 获得什么保证，以及保证何时成立');
notes(s,'架构取舍对照',1.5,'本表是对过去一周设计讨论的定性归纳，不是测量排名。必须在相同接受层级和可比假设下比较。A 的区域确认在旧实现中仍可能属于跨区可截断 preview；B 明确把强保证放到全局层；C 要先证明区域封装契约；D 以专门状态机制替代通用外层共识，但其控制信任、cutoff 和数据持久性义务都需要写明。不能仅凭多签名称宣称去掉了共识成本。',['research-index/2026-09-28-weekly-design/tradeoffs-zh.md']);
}
function sixty(){return 60;}
// 12: Closing architectural decisions, with remaining work visible.
{
const s=base('当前选择与需要闭合的接口');
text(s,'当前选择',69,175,526,43,31,C.teal,true);
text(s,'区域内持续生产与批量确认',69,247,535,48,30);
text(s,'正常交班等待 finalized anchor',69,324,535,66,30);
text(s,'委员会只维护高度与区域状态',69,428,535,65,30);
line(s,650,176,650,552);
text(s,'仍需闭合',708,175,495,43,31,C.amber,true);
text(s,'跨区继承全部已确认前缀',708,247,495,48,30);
text(s,'事故截止、数据恢复与旧权限失效',708,324,495,83,30);
text(s,'L1 对源流与轮换的验证关系',708,428,495,65,30);
takeaway(s,'设计重点：常态出流与日常交班的低成本',C.blue);
notes(s,'当前选择与剩余接口',1,'收束为本周已经接受的取舍：区域内优化持续 pipeline，batch 摊薄成本；正常交班接受旧 anchor final 后再开新区；委员会限制业务范围。成本并未消失，跨区前缀继承、事故恢复和 L1 验证关系仍必须闭合。接下来的协议部分会解释这些接口如何实现，不在这一组架构 slides 中展开算法。',['formalization/proposals/v3-status-entry-2026-09-28/outline-and-revision-plan-zh.md','research-index/2026-09-28-weekly-design/tradeoffs-zh.md']);
}

await fs.mkdir(path.join(BUILD,VERSION),{recursive:true});
const candidatePath=path.join(BUILD,VERSION,'candidate.pptx');
await (await PresentationFile.exportPptx(p)).save(candidatePath);
await fs.writeFile(path.join(BUILD,VERSION,'presentation.json'),JSON.stringify(p.toProto()));
await fs.writeFile(path.join(BUILD,VERSION,'story.json'),JSON.stringify(metadata,null,2));
for(let i=0;i<p.slides.items.length;i++){
 const s=p.slides.items[i];const b=await p.export({slide:s,format:'png',scale:1.4});
 await fs.writeFile(path.join(BUILD,VERSION,`slide-${String(i+1).padStart(2,'0')}.png`),new Uint8Array(await b.arrayBuffer()));
 const l=await s.export({format:'layout'});await fs.writeFile(path.join(BUILD,VERSION,`slide-${String(i+1).padStart(2,'0')}.layout.json`),await l.text());
}
const finalPath=path.join(WORK,'output','sequencer-streaming-design-final.pptx');
const result=await finalizePresentation({workspaceDir:WORK,candidatePath,finalPath,pythonExecutable:PYTHON,integrityValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_package_integrity.py'),layoutValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_layout_geometry.py'),layoutArgs:['--expected-slide-size-emu','12192000,6858000','--validate-bullet-geometry','--validate-heading-fit','--require-native-table-slide','11'],requiredNativeTableOwnerSlides:[11],fontPolicy,verifyArtifactToolImport:true,receiptPath:path.join(BUILD,VERSION,'validation.json')});
console.log(JSON.stringify({finalPath,slides:metadata.length,minutes:metadata.reduce((a,b)=>a+b.minutes,0),result}));
