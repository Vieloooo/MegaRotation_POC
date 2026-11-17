"use strict";

const data = JSON.parse(document.getElementById("replay-data").textContent);
const byId = (id) => document.getElementById(id);
const number = new Intl.NumberFormat("zh-CN");
const formatUs = (value) => `${number.format(value)} μs`;
const seconds = (value) => (value / 1_000_000).toFixed(3) + " s";
const escape = (value) => String(value).replace(/[&<>"']/g, (ch) => ({
  "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;",
})[ch]);
const code = (value) => `<code>${escape(value)}</code>`;
const windowLabel = (d) => `[${d.window[0]}, ${d.window[1]}]`;
const contextLabel = (d) => `R${d.region} · β${d.beta}/r${d.session}`;
const percent = (tick) => `${100 * tick / data.end_us}%`;
const contexts = new Map(data.contexts.map((d) => [d.cid, d]));
const isForce = Boolean(data.force);
const projectedL1 = data.l1.filter((row) => row.finalized_us <= data.end_us);

const lanes = [
  ["l1", isForce ? "Anvil L1" : "L1 fixture", isForce ? forceL1Events() : projectedL1.map((row) => ({
    tick: row.finalized_us, seq: 0, index: row.height, label: `ℓ${row.height}`,
    title: `finalized L1 fixture header ℓ${row.height}`,
  }))],
  ["espresso", "Espresso 公共日志", data.entries.map((entry) => ({
    tick: data.espresso[entry.q].finalized_us, seq: entry.publisher.seq, index: entry.q,
    label: `q${entry.q}`, title: `Espresso finalized q${entry.q}`,
  }))],
  ["regional", "区域最终区块", data.entries.map((entry, index) => ({
    tick: entry.deliveries[0].tick_us, seq: entry.deliveries[0].seq, index,
    label: `h${entry.body.height}`, title: `区域最终区块 h${entry.body.height}`,
  }))],
  ["g", "G 链接纳", data.entries.map((entry, index) => ({
    tick: entry.replay.tick_us, seq: entry.replay.seq, index,
    label: entry.excluded ? "q10 ×" : `${entry.body.height}${entry.handoff ? "↗" : ""}`,
    stage: entry.excluded ? "excluded" : entry.handoff ? "regular" : isForce && index === data.entries.length - 1 ? "h16" : undefined,
    level: entry.excluded ? 2 : undefined,
    title: entry.excluded ? "q10 完成重放 · 旧 physical h11 被排除" :
      `G 接纳 h${entry.body.height} / q${entry.q}${entry.handoff ? " 并接棒" : ""}`,
    handoff: Boolean(entry.handoff), excluded: Boolean(entry.excluded),
  }))],
];
if (isForce) {
  lanes[3][2].push({
    tick: data.force.replay.tick_us, seq: data.force.replay.seq, index: data.entries.length,
    label: "Force", title: "Force 激活 · β0 → β1", level: 0, stage: "force", handoff: true,
  });
}
const frames = [{ kind: "start", tick: 0, seq: 0, index: 0 }].concat(
  lanes.flatMap(([kind, , events]) => events.map((event) => ({ ...event, kind })))
    .sort((a, b) => a.tick - b.tick || a.seq - b.seq),
);
const frameByStage = new Map(frames.flatMap((frame, i) => frame.stage ? [[frame.stage, i]] : []));
const frameByEvent = new Map(frames.map((frame, i) => [`${frame.kind}:${frame.index}`, i]));
let cursor = 0;
let playback = null;
let state = null;

function metricRows(rows, className = "facts") {
  return `<dl class="${className}">${rows.map(([key, value]) =>
    `<div><dt>${key}</dt><dd>${value}</dd></div>`).join("")}</dl>`;
}

function trace(ref) {
  return code(`${data.source.trace_file}:${ref.line} · seq ${ref.seq} · ${ref.kind}`);
}

function positionRef(q, index) {
  return code(`${data.espresso[q].file} · messages[${index}]`);
}

function evidence(items, label = "定位原始记录与独立证据") {
  return `<details><summary>${label}</summary><ul class="evidence">${items
    .map(([key, value]) => `<li><span>${key}</span>${value}</li>`).join("")}</ul></details>`;
}

function title(head, description, tag) {
  return `<div class="inspector-head"><div><h2 id="selection-title">${head}</h2>
    <p>${description}</p></div><span class="tag">${tag}</span></div>`;
}

function proofEvidence(entry) {
  const proof = data.espresso[entry.proof.q].messages[entry.proof.message];
  const expected = entry.excluded ? "late[0]" : `expected[${entry.oracle.expected_index}]`;
  return [
    ["公开最终证明", positionRef(entry.proof.q, entry.proof.message)],
    ["证明字节", code(proof.proof)],
    ["独立核对", code(`public-replay.json · ${expected} · public_finals[${entry.oracle.proof_index}]`)],
    ...entry.ancestry.map((body) => [`h${body.height} 首次公开正文`, positionRef(body.q, body.message)]),
  ];
}

function linkButton(kind, index, label, inspectOnly = false) {
  return `<button type="button" data-reference="${kind}:${index}"${inspectOnly ? " data-inspect=\"true\"" : ""}>${label}</button>`;
}

function l1Details(row) {
  const inRange = row.finalized_us <= data.end_us;
  return title(`L1 fixture · ℓ${row.height}`,
    inRange ? "来自封存 fixture 的 finalized header" : "来自封存 fixture，finalized 时刻超出本次轨迹",
    "fixture 记录")
    + metricRows([
      ["Header hash", code(row.hash)],
      ["Parent hash", code(row.parent)],
      ["L1 timestamp", formatUs(row.timestamp_us)],
      ["fixture finalized τ", formatUs(row.finalized_us)],
      ["Reader 收取 τ", "未记录"],
      ["控制列表", code("[]") + "（该 fixture 的空控制承诺）"],
    ])
    + evidence([
      ["L1 header", code(`raw/l1.bin · rows[${row.height}]`)],
      ["空控制承诺", code(row.control_commitment)],
      ["文件 SHA-256", code(data.source.input_sha256["raw/l1.bin"])],
    ]);
}

function espressoDetails(entry) {
  const row = data.espresso[entry.q];
  return title(`Espresso 公共日志 · q${row.q}`, ` finalized 前缀 J₀…J${row.q}`, "原始日志 + trace")
    + metricRows([
      ["公开时间键 θ", formatUs(row.key_us)],
      ["proposal τ", formatUs(row.proposal_us)],
      ["finalized τ", formatUs(row.finalized_us)],
      ["结果 q_avail", `q${entry.q_avail} · h${entry.body.height}`],
      ["来源", code(row.source)],
    ])
    + `<div class="trace-links">${linkButton("g", entry.q, entry.excluded ? "查看旧结果排除" : `查看 h${entry.body.height} 接纳`)}</div>`
    + evidence([
      ["PELG 文件", code(row.file)],
      ["PublisherRecorded", trace(entry.publisher)],
      ...row.messages.map((msg) => msg.kind === "body"
        ? [`本条日志正文 h${msg.body.height}`, positionRef(row.q, msg.index) + " " + code(msg.body.physical)]
        : ["本条日志最终证明", positionRef(row.q, msg.index)]),
      ...proofEvidence(entry),
    ]);
}

function regionalDetails(entry) {
  const body = entry.body;
  const d = contexts.get(entry.cid);
  const seen = entry.deliveries.filter((r) => r.tick_us < state.tick ||
    (r.tick_us === state.tick && r.seq <= state.seq));
  return title(`区域最终区块 · R${d.region} / h${body.height}`,
    "同一 (cid, physical_id) 的交付记录合并为一个区块", "H3RS + trace")
    + metricRows([
      ["首条交付 τ", formatUs(entry.deliveries[0].tick_us)],
      ["截至游标的交付记录", `${seen.length} / 5`],
      ["物理区块", code(body.physical)],
      ["物理父块", code(body.parent)],
      ["Descriptor", contextLabel(d) + " · 窗口 " + windowLabel(d) + "<br>" + code(d.cid)],
      ["连接 anchor", `h${d.anchor.height} · ` + code(d.anchor.physical)],
      ["L2 timestamp / origin", `${formatUs(body.timestamp_us)} · ℓ${body.origin}`],
    ])
    + evidence([
      ["区块正文、最终证明与路径", code(entry.result_file) + " · H3RS target / final_proof / chain"],
      ["文件 SHA-256", code(data.source.input_sha256[entry.result_file])],
      ...seen.map((r) => [`pid ${r.pid} · ${formatUs(r.tick_us)}`, trace(r)]),
      ...(state.espresso >= entry.q ? proofEvidence(entry) : []),
    ]);
}

function handoffBox(entry) {
  if (!entry.handoff) return "";
  const event = entry.handoff;
  const next = contexts.get(event.new);
  const old = contexts.get(event.old);
  return `<div class="handoff-box">
    <div><strong>R${old.region} → R${next.region}</strong>
      <span>h${entry.body.height} 达到窗口上界</span>
      <small>SessionClosed(COMPLETE) + SessionAssigned</small></div>
    <div><strong>后继窗口 ${windowLabel(next)}</strong>
      <span>${contextLabel(next)}</span>
      <small>前驱 h_max + 1 至 h_max + TB</small></div>
    <div><strong>连接 anchor h${event.anchor.height}</strong>
      <span>Plan reference h${event.reference.height}</span>
      <small>前驱末端用于连接，前驱初始 anchor 用于 Plan</small></div>
  </div>`;
}

function gDetails(entry) {
  const d = contexts.get(entry.cid);
  const body = entry.body;
  const check = entry.coverage;
  const cover = data.l1[check.l1_control_height];
  const publicKey = data.espresso[entry.q_avail].key_us;
  const judgeKey = entry.position.key_us;
  const l1Observation = isForce
    ? `模拟 finalized τ ${formatUs(cover.finalized_us)}，reader A 已读 τ ${formatUs(cover.readers.A.trace.tick_us)}。`
    : `fixture finalized τ ${formatUs(cover.finalized_us)}，reader 收取时刻未记录。`;
  const firstExpiry = entry.earliest_expiry_us ?? check.expiry_us;
  const parentHeight = entry.append[0] - 1;
  const controls = [
    ["Espresso 覆盖 · §3.4", `处理 q${entry.q} 时，finalized J₀…J${entry.q} 覆盖 x = ${formatUs(judgeKey)}。`],
    ["L1 控制覆盖 · §3.4", `x − Mₐ (${formatUs(data.profile.announcement_us)}) = ${formatUs(check.threshold_us)}。首个满足 ts ≥ x − Mₐ 的 header 是 ℓ${cover.height}，ts ${formatUs(cover.timestamp_us)}。${l1Observation}`],
    ["职责与窗口 · §4.2", `${contextLabel(d)} 当时为 NEXT，匹配当前 cid。新增 physical h${body.height} 位于 ${windowLabel(d)}。`],
    ["首次公开完整 · §3.2", `经独立报告验证的最终证明 ${code(`(${entry.proof.q}, ${entry.proof.message})`)} 与 ${entry.ancestry.map((row) => `h${row.height}`).join("、")} 的 physical 正文共同确定 q_avail = q${entry.q_avail}。`],
    ["父链与 anchor · §4.2", `本块物理父块匹配 G 的 h${parentHeight}，完整 physical 路径从选定 anchor h${d.anchor.height} 接出。`],
    ["原生验证 · §4.2", `L2 timestamp ${formatUs(body.timestamp_us)}。使用 finalized origin ℓ${check.l1_origin_height} 和 anchor origin ℓ${check.l1_anchor_origin_height}，origin 未超前，drift ${formatUs(check.drift_us)} ≤ ${formatUs(data.config.t_drift_us)}。强制前缀由 L1 attributes 与 DEP fixture 的 ${check.dep.length} 条 credit 组成。当前 Plan reference origin 是 ℓ${check.l1_reference_origin_height}。`],
    ["公开截止 · §4.2", `θ(q_avail) ${formatUs(publicKey)} < SessionEnd ${formatUs(check.session_end_us)}。`],
    ["整段 Expire · §4.2", `追加区间 [${entry.append.join(", ")}] 的逐高度检查通过。x ${formatUs(judgeKey)} < 最早过期键 ts*(h${entry.append[0]}) + W_exp = ${formatUs(firstExpiry)}。`],
  ];
  if (entry.padding?.length) {
    const previous = entry.padding.at(-1);
    controls.push(["虚拟前态 · §4.2", `physical h${body.height} 从 padding h${previous.height} 的执行后状态继续执行，物理父块仍连接 h${parentHeight}。`]);
  }
  if (entry.handoff) {
    const next = contexts.get(entry.handoff.new);
    const origin = next.reference.origin;
    controls.push(["满窗 Regular · §4.3",
      `同一 Espresso(q${entry.q}) 转移关闭旧职责。生产 anchor 使用 h${entry.handoff.anchor.height}，Plan reference 使用 h${entry.handoff.reference.height}（origin ℓ${origin}）。β 不变，r 加一。`]);
  }
  const refs = [
    ["区域结果", code(entry.result_file)],
    ["PublisherRecorded", trace(entry.publisher)],
    ["GAccepted", trace(entry.accept)],
    ["GfrReplayRecord", trace(entry.replay)],
    ["重放记录 ID", code(entry.record_id)],
    ["L1 coverage / origin", isForce
      ? code(`${data.control.rpc_file} · blocks[${cover.height}] / blocks[${check.l1_origin_height}]`)
      : code(`raw/l1.bin · rows[${cover.height}] / rows[${check.l1_origin_height}]`)],
    ["覆盖与时间参数", code("raw/config.bin · raw/profile.json · raw/dep.bin")],
    ...proofEvidence(entry),
  ];
  if (entry.handoff) {
    refs.push(
      ["Handoff", trace(entry.handoff.trace)],
      ["后继 descriptor", code(entry.handoff.descriptor_file)],
      ["Lifecycle event IDs", entry.handoff.event_ids.map(code).join(" / ")],
      ["新 cid", code(entry.handoff.new)],
      ["连接 anchor", code(entry.handoff.anchor.physical)],
      ["后继 Plan reference", code(entry.handoff.reference.physical)],
    );
  }
  const facts = metricRows([
    ["接纳观察 τ", formatUs(entry.accept.tick_us)],
    ["公共位置", code(`Espresso(${entry.q}) · x = ${formatUs(judgeKey)}`)],
    ["首次完整公开", code(`q_avail = ${entry.q_avail} · θ = ${formatUs(publicKey)}`)],
    ["新增 G 区间", code(`[${entry.append.join(", ")}]`) + ` · R${d.region}`],
    ["当前结果 cid", code(entry.cid)],
    ["物理区块", code(body.physical)],
    ["物理父块", code(body.parent)],
    ["选定 anchor", `h${d.anchor.height} · ` + code(d.anchor.physical)],
    ["G native hash", code(entry.native)],
    ["执行后 state root", code(entry.state_root)],
  ]);
  return title(`G 接纳 · h${body.height} / q${entry.q}`,
    entry.handoff ? "接纳与职责切换属于同一公共重放位置" : `R${d.region} 的结果扩展已接纳前缀`,
    entry.handoff ? "满窗 Regular" : "GAccepted")
    + handoffBox(entry)
    + `<div class="explanation"><div><h3>封存记录观察</h3>${facts}</div>
    <div><h3>按规则核对</h3>${metricRows(controls, "guards")}</div></div>`
    + `<div class="trace-links">
       ${linkButton(isForce ? "l1-header" : "l1", cover.height, `L1 覆盖 ℓ${cover.height}`, true)}
       ${linkButton(isForce ? "l1-header" : "l1", check.l1_origin_height, `L1 origin ℓ${check.l1_origin_height}`, true)}
       ${linkButton("espresso", entry.q, `Espresso q${entry.q}`, true)}
       ${linkButton("regional", entry.q, `区域 h${body.height}`, true)}
       </div>`
    + (isForce ? paddingDetails(entry) : "")
    + evidence(refs);
}

function inspect(frame) {
  const entry = data.entries[frame.index];
  const views = {
    l1: () => isForce ? (frame.stage ? forceL1Stage(frame) : forceL1Details(data.l1[frame.index]))
      : l1Details(data.l1[frame.index]),
    "l1-header": () => forceL1Details(data.l1[frame.index]),
    "l1-fixture": () => l1Details(data.worker_l1[frame.index]),
    espresso: () => espressoDetails(entry),
    regional: () => regionalDetails(entry),
    g: () => frame.stage === "force" ? forceDetails() : entry.excluded ? exclusionDetails(entry) : gDetails(entry),
    start: () => title("初始职责", "G 从封存 genesis 开始", "Genesis") + metricRows([
      ["选定计划", contextLabel(data.contexts[0]) + " · 窗口 " + windowLabel(data.contexts[0])],
      ["初始 anchor", "h0 · " + code(data.config.genesis.physical)],
      ["初始 cid", code(data.contexts[0].cid)],
    ]) + evidence([["Config / descriptor", code("raw/config.bin · raw/genesis-descriptor.bin")]]),
  };
  byId("inspector").innerHTML = views[frame.kind]();
  for (const marker of document.querySelectorAll(".mark")) {
    marker.setAttribute("aria-current", String(Number(marker.dataset.frame) ===
      frameByEvent.get(`${frame.kind}:${frame.index}`)));
  }
}

function makePlot() {
  byId("timeline").insertAdjacentHTML("beforebegin", '<p class="timeline-legend"><span class="legend-dot"></span>实心点表示该链新增或替换区块，悬停查看高度，点击查看事件。</p>');
  const axes = [0, 5, 10, 15, 20, 25, 30].filter((x) => x * 1_000_000 <= data.end_us);
  const plot = (label, axisName, classname, events) =>
    `<div class="timeline-row${isForce && (classname === "l1" || classname === "g") ? " stacked" : ""}" aria-label="${axisName}" data-lane="${classname}">
      <div class="lane-name"><span>${label}</span><span class="lane-head mono"></span></div>
      <div class="plot">${events ? '<span class="track"></span>' : ""}
        ${axes.map((x) => `<span class="${events ? "guide" : "ruler-label"}" style="left:${percent(x * 1_000_000)}">${events ? "" : x + "s"}</span>`).join("")}
        <span class="cursor" style="left:0%"></span>
        ${events ? events.map((event) =>
          `<button type="button" class="mark ${classname}${event.handoff ? " handoff" : ""}${event.excluded ? " excluded" : ""}${classname === "g" && !event.excluded && event.stage !== "force" ? " block-accepted" : ""}" style="left:${percent(event.tick)}${event.level === undefined ? "" : `;top:${18 + 32 * event.level}%`}"
             data-frame="${frameByEvent.get(`${classname}:${event.index}`)}"
             title="${escape(event.title)} · ${seconds(event.tick)}">${event.label}</button>`).join("") : ""}
      </div></div>`;
  byId("timeline").innerHTML = plot("虚拟时间 τ", "刻度", "ruler")
    + lanes.map(([kind, label, events]) => plot(label, label, kind, events)).join("");
}

function followCursor(tick) {
  const scroller = document.querySelector(".timeline-scroll");
  if (scroller.scrollWidth <= scroller.clientWidth) return;
  const track = document.querySelector('[data-lane="ruler"] .plot');
  const x = track.getBoundingClientRect().left - scroller.getBoundingClientRect().left
    + scroller.scrollLeft + track.clientWidth * tick / data.end_us;
  const labelWidth = track.previousElementSibling.offsetWidth;
  if (x > scroller.scrollLeft + scroller.clientWidth - 24 ||
      x < scroller.scrollLeft + labelWidth + 14) {
    scroller.scrollLeft = x - scroller.clientWidth * 0.68;
  }
}

function seek(step) {
  cursor = Math.max(0, Math.min(step, frames.length - 1));
  const frame = frames[cursor];
  const latest = {};
  let active = data.contexts[0];
  let head = null;
  let pc = null;
  for (let i = 1; i <= cursor; i++) {
    const current = frames[i];
    latest[current.kind] = current.index;
    if (current.kind === "g") {
      if (current.stage === "force") {
        active = contexts.get(data.force.new);
        pc = data.force.position;
      } else {
        const entry = data.entries[current.index];
        pc = entry.position;
        if (!entry.excluded) head = entry;
        if (entry.handoff) active = contexts.get(entry.handoff.new);
      }
    }
  }
  state = { tick: frame.tick, seq: frame.seq, espresso: latest.espresso ?? -1 };
  byId("clock").textContent = seconds(frame.tick);
  byId("head").textContent = head ? `h${head.body.height}` : "h0";
  byId("context").textContent = `${contextLabel(active)} ${windowLabel(active)}`;
  const pcName = pc?.kind === "Force" ? `Force(${data.control.position.join(",")})` : `q${pc?.index.Espresso}`;
  byId("position").textContent = pc ? `${pcName} · ${seconds(pc.key_us)}` : "Genesis";
  byId("scrub").value = cursor;
  byId("step-label").textContent = `${cursor} / ${frames.length - 1}`;
  byId("reset").disabled = cursor === 0;
  byId("previous").disabled = cursor === 0;
  byId("next").disabled = cursor === frames.length - 1;
  for (const line of document.querySelectorAll(".cursor")) line.style.left = percent(frame.tick);
  followCursor(frame.tick);
  for (const [kind, , events] of lanes) {
    const lane = document.querySelector(`[data-lane="${kind}"]`);
    lane.querySelector(".lane-head").textContent = events.find((event) => event.index === latest[kind])?.label ?? "—";
  }
  for (const marker of document.querySelectorAll(".mark")) {
    marker.hidden = Number(marker.dataset.frame) > cursor;
  }
  for (const jump of document.querySelectorAll("[data-jump]")) {
    jump.setAttribute("aria-current", String(frame.kind === "g" && +jump.dataset.jump === frame.index + 1));
  }
  for (const jump of document.querySelectorAll("[data-stage]")) {
    jump.setAttribute("aria-current", String(frame.stage === jump.dataset.stage));
  }
  inspect(frame);
}

function pause() {
  if (playback !== null) clearTimeout(playback);
  playback = null;
  byId("play").textContent = "▶ 播放";
  byId("play").setAttribute("aria-pressed", "false");
}

function queueNext() {
  if (cursor === frames.length - 1) {
    pause();
    return;
  }
  const gap = frames[cursor + 1].tick - frames[cursor].tick;
  playback = setTimeout(() => {
    seek(cursor + 1);
    queueNext();
  }, Math.max(160, Math.min(950, gap / 2500)));
}

function jump(step) {
  pause();
  seek(step);
}

function fixtureTable() {
  const table = (isForce ? data.worker_l1 : data.l1).map((row) => `<tr>
      <td>${linkButton(isForce ? "l1-fixture" : "l1", row.height, `ℓ${row.height}`, true)}</td>
      <td>${formatUs(row.timestamp_us)}</td><td>${formatUs(row.finalized_us)}</td>
      <td>${row.finalized_us <= data.end_us ? "轨迹时段内" : "超出轨迹"}</td>
      <td>${code("[]")}</td><td>${code(row.hash)}</td>
    </tr>`).join("");
  byId("fixture-body").innerHTML = `<div class="table-wrap"><table>
    <thead><tr><th scope="col">Header</th><th scope="col">Timestamp</th>
    <th scope="col">Fixture finalized τ</th><th scope="col">时段</th>
    <th scope="col">控制</th><th scope="col">Hash</th></tr></thead><tbody>${table}</tbody>
    </table></div>`;
}

function sourceSummary() {
  byId("source").innerHTML = metricRows([
    [isForce ? "M2 原始证据包" : "M1 原始证据包", code(data.source.bundle) + " · seed " + data.source.seed],
    ["结果", isForce ? "11 个 physical 接纳 · 5 个 padding · 12 份独立核对的最终证明" : `21 次 G 接纳 · 2 次满窗交接 · 21 份独立核对的最终证明`],
    ["覆盖范围", isForce ? `封存虚拟时间截至 ${formatUs(data.end_us)}，G 到 h16。` : `封存虚拟时间截至 ${formatUs(data.end_us)}，G 到 h21，L1 fixture 在此时段内显示至 ℓ${projectedL1.at(-1).height}。`],
  ]) + evidence([
    ["运行提交", code(data.source.revision)],
    ["manifest.json · SHA-256", code(data.source.manifest_sha256)],
    ["独立证据包", code(data.source.independent)],
    ["public-replay.json · SHA-256", code(data.source.public_replay_sha256)],
    ...(isForce ? [
      ["report.json · SHA-256", code(data.source.report_sha256)],
      ["controls.json · SHA-256", code(data.source.controls_sha256)],
    ] : []),
    ["结束事件", trace(data.completion)],
    ...Object.entries(data.source.input_sha256).map(([file, digest]) => [code(file), code(digest)]),
  ], "提交、输入文件与 SHA-256");
}

if (isForce) {
  document.title = "M2 Healthy Force · GFR 回放";
  document.querySelector("h1").textContent = "Healthy Force 回放";
  document.querySelector(".eyebrow").textContent = "SEQUENCER ROTATION POC / M2";
  document.querySelector(".subtitle").textContent = "跟随真实 L1 控制，查看公开日志如何切换 GFR 职责";
  document.querySelector(".source-jump span").textContent = `${data.source.revision.slice(0, 8)} · seed ${data.source.seed}`;
  document.querySelector("#fixture summary").textContent = "Worker metadata fixture · 31 个 header";
  document.querySelector("#scope").textContent = "Anvil L1 的 header、控制与 reader 交付来自实际封存日志。raw/l1.bin 是 worker 执行所用的高度和时间戳投影。";
  document.querySelector("#timing-note").textContent = "同一虚拟时刻的 trace 按 seq 展示。每条 GfrReplayRecord 是一个原子公共转移。";
  document.querySelector("#rule-source").textContent = "规则来源为 formalization_v10/09_finality_rules_zh_v0_1.md §3.2–3.4、§4.2–4.4。先按公开时间键排序，同键时 L1 control → Expire → SessionEnd → Espresso，同类再按各自索引。";
  document.querySelector(".jump-list").innerHTML = [
    ["submit", "Force 提交"], ["include", "ℓ6 收录"], ["finalize", "模拟 finalized"],
    ["reader-a", "Reader A"], ["reader-b", "Reader B"], ["regular", "R0 → R1"],
    ["force", "Force 激活"], ["excluded", "排除旧 h11"], ["h16", "接纳 R2 h16"],
  ].map(([stage, label]) => `<button type="button" data-stage="${stage}">${label}</button>`).join("");
  document.querySelector(".scenario-link").href = "index.html";
  document.querySelector(".scenario-link").textContent = "M1 Regular →";
}
makePlot();
fixtureTable();
sourceSummary();
byId("scrub").max = frames.length - 1;
byId("scrub").addEventListener("input", (event) => jump(Number(event.target.value)));
byId("reset").addEventListener("click", () => jump(0));
byId("previous").addEventListener("click", () => jump(cursor - 1));
byId("next").addEventListener("click", () => jump(cursor + 1));
byId("play").addEventListener("click", () => {
  if (playback !== null) {
    pause();
    return;
  }
  if (cursor === frames.length - 1) seek(0);
  byId("play").textContent = "Ⅱ 暂停";
  byId("play").setAttribute("aria-pressed", "true");
  queueNext();
});
byId("filters").addEventListener("change", (event) => {
  document.querySelector(`[data-lane="${event.target.dataset.filter}"]`).hidden = !event.target.checked;
});
document.querySelector("main").addEventListener("click", (event) => {
  const target = event.target.closest("button");
  if (!target) return;
  if (target.dataset.stage) jump(frameByStage.get(target.dataset.stage));
  if (target.dataset.jump) jump(frameByEvent.get(`g:${Number(target.dataset.jump) - 1}`));
  if (target.dataset.frame) jump(Number(target.dataset.frame));
  if (target.dataset.reference) {
    const step = frameByEvent.get(target.dataset.reference);
    const [kind, index] = target.dataset.reference.split(":");
    if (target.dataset.inspect) {
      pause();
      inspect({ kind, index: Number(index) });
      if (target.closest("#fixture")) {
        byId("inspector").scrollIntoView({ block: "start" });
        byId("selection-title").setAttribute("tabindex", "-1");
        byId("selection-title").focus({ preventScroll: true });
      }
    } else if (step !== undefined) {
      jump(step);
    }
  }
});
seek(isForce ? frameByStage.get("submit") : frameByEvent.get("g:9"));
