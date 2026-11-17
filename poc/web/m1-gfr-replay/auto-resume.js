"use strict";

const datasets = JSON.parse(document.querySelector("#replay-data").textContent);
let data, frames, last, end;
const lanes = [["l1", "L1 / Inbox"], ["da", "DA"], ["d", "D 派生"], ["gprime", "G′ 物化"],
               ["espresso", "Espresso"], ["regional", "Replica / 本地时钟"], ["g", "G / GFR"],
               ["governance", "治理"]];
const stages = [
  ["fault", "排序故障"], ["frozen-q", "q 增而 θ 冻结"], ["d-advance", "D 前进"],
  ["local-stop", "本地 stop"], ["wait", "公共 WAIT"], ["recovery", "排序恢复"],
  ["expire", "公开 Expire / HALT"], ["eligible", "可签证据"], ["target", "求解目标"],
  ["signed", "治理签名包"], ["prepared", "持久准备"], ["submit", "广播"],
  ["included", "L1 收录"], ["cut", "历史切面"], ["gprime-cut", "G′ 对齐"],
  ["resume", "RESUME 激活"], ["ready", "Ready"], ["enter", "Enter"], ["new-result", "新区域 Final"],
  ["publish", "公开完整"], ["accepted", "首块进入 G"],
];
const labels = {
  worker: "Replica", clock_us: "本地时钟 Cₘ", stop_us: "本地 stop 阈值", session_end_key_us: "公共 SessionEnd 键",
  same_key_order: "同键顺序", proposal_tick_us: "提议虚拟时点", finalized_tick_us: "finalized 虚拟时点",
  h_guard: "H_guard", guard_parts: "推进上界分量", heads: "G / D / Inbox heads",
  inclusion: "预计 L1 收录区间", target_timestamp_us: "目标 L2 timestamp",
  lower_timestamp_us: "timestamp 下界", upper_timestamp_us: "timestamp 上界",
  minimum_height: "整数高度下界", maximum_height: "整数高度上界",
  paths_ready: "恢复路径就绪", clock_ready: "时钟与 lease 就绪", d_drained: "D 已处理到可用边界",
  t_eligible_us: "首个可签 quorum", submit_delay_us: "提交耗时", within_budget: "提交耗时在预算内",
  within_inclusion: "实际收录在原区间内", act_us: "实际公开 act", request: "请求 digest",
  published: "公布 Espresso tip", verified: "已验证 Espresso tip",
  published_l1: "公布 L1 tip", verified_l1: "已验证 L1 tip", pc: "本地已完成公开位置",
  observed_us: "观察时间", eligible: "可签", pending: "待处理控制", validated_q: "本地完整 q",
  position: "位置", transaction_position: "L1 交易位置", cursor_position: "CursorSet L1 位置",
  resume_position: "ResumeRecorded L1 位置", receipt_log_index: "RPC 区块内 logIndex",
  cursor_rpc_log_index: "CursorSet RPC logIndex", timestamp_us: "L1 timestamp", theta_us: "公开键 θ",
  tx: "交易", status: "Receipt status", target: "恢复目标", previous: "上次观察 / 旧 cursor", next: "新 cursor",
  commitment: "DA commitment", verified_commitment: "已校验 commitment", gpos: "认证源 G 位置",
  cut_id: "ResumeCut 摘要", control_seq: "控制序号", signed_origin: "请求 origin",
  policy: "控制 policy", proof: "最终证明标识", q_avail: "公开完整位置 q_avail",
  missing: "等待原因", reason: "派生或处理原因", coverage: "GFR 覆盖", lifecycle: "生命周期顺序",
  closed_cids: "终止的旧 cid", before: "旧 G", after: "新 G", retained: "保留至",
  branch: "分支", r: "Session 序号", region: "地区", window: "职责窗口", floor: "有效 L1 floor",
  source_region: "来源地区", plan_height: "Plan 查询高度", ref: "Plan Ref 连接 ID",
  native: "Native hash", native_parent: "Native parent", physical: "Physical ID",
  physical_parent: "Physical parent", origin: "实际 L1 origin", pre: "执行前 root", post: "执行后 root",
  ordinary_count: "普通交易数", id: "派生 anchor 连接 ID", height: "高度",
  plan_header_hash: "Plan 已验证的 L1 header", window_end: "窗口截止",
  first_diff: "首个不同高度", common_height: "共同高度", common_prefix: "共同原生前缀",
};
const kindNames = {genesis: "Genesis", regional: "区域块", batch: "普通批次", "window-fallback": "窗口回退 · deposit-only",
                   "cursor-fill": "CursorSet 填充 · deposit-only", substitution: "同高替换", padding: "Virtual padding"};
const byId = id => document.getElementById(id);
const time = us => us == null ? "—" : (us / 1000000).toFixed(6) + " s";
const pct = us => us * 100 / end;
let cursor = 0;
let timer;
let fixedHeight = null;

function esc(value) {
  const text = value == null ? "—" : typeof value === "object" ? JSON.stringify(value) : String(value);
  return text.replace(/[&<>"']/g, c => ({"&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;"})[c]);
}

function position(value) {
  if (!value) return "Genesis";
  return `${value.kind} ${value.index.join(",")} · ${time(value.key_us)}`;
}

function fields(rows) {
  return rows.map(([label, value]) =>
    `<div><dt>${esc(labels[label] || label)}</dt><dd><code>${esc(value)}</code></dd></div>`).join("");
}

function pretty(obj, omit = new Set()) {
  return Object.entries(obj).filter(([name]) => !omit.has(name)).map(([name, value]) => {
    if (["position", "gpos", "pc"].includes(name) && value?.key_us != null) return [name, position(value)];
    if (name.endsWith("_us")) return [name, time(value)];
    if (name === "inclusion" && Array.isArray(value)) return [name, value.map(time).join(" 至 ")];
    return [name, value];
  });
}

function blockFacts(block) {
  return fields(pretty(block).map(([field, value]) => [field, field === "kind" ? kindNames[value] : value]));
}

function tailTable(list) {
  if (!list.length) return '<p class="muted empty">空</p>';
  return `<div class="table-wrap"><table><thead><tr><th>高度</th><th>Native hash</th><th>来源</th><th>执行后 root</th></tr></thead>
    <tbody>${list.map(block => `<tr><td>h${block.height}</td><td><code>${esc(block.native)}</code></td>
      <td>${esc(block.provenance)} · ${esc(kindNames[block.kind])}</td><td><code>${esc(block.post)}</code></td></tr>`).join("")}</tbody></table></div>`;
}

function inspect(frame) {
  const {detail, change, kind} = frame;
  const omit = new Set(["block", "blocks", "before", "after", "revoked", "adopted", "fill", "prefix", "anchor", "duty"]);
  const parts = [`<dl class="facts">${fields(pretty(detail, omit))}</dl>`];
  if (kind === "signer_view") {
    const view = frame.signer_views[detail.signer];
    if (view.solver) parts.push(`<details open><summary>整数目标求解</summary>
      <dl class="facts">${fields([...pretty(view.solver), ["所选目标", view.target ?? "等待 · 无可行目标"]])}</dl></details>`);
  }
  if (detail.members) parts.push(`<details open><summary>入包地址、签署时点与 digest</summary>${
    detail.members.map(member => `<dl class="facts">${fields(pretty(member))}</dl>`).join("")}</details>`);
  if (detail.reconstructed) parts.unshift('<p class="subtitle">G′ 按 Inbox 接纳来源与历史切面重建。</p>');
  if (detail.header) parts.push(`<details open><summary>收录坐标</summary><dl class="facts">${fields(pretty(detail.header))}</dl></details>`);
  if (detail.anchor) parts.push(`<details open><summary>目标 anchor 与历史派生切面</summary>
    <dl class="facts">${fields(pretty(detail.anchor))}</dl>
    <p class="subtitle">采纳到 h${detail.target}。此位置保存的 D 前缀完整到 h${detail.prefix.length - 1}。</p></details>`);
  if (detail.block) parts.push(`<details open><summary>区块与父状态</summary><dl class="facts">${blockFacts(detail.block)}</dl></details>`);
  if (detail.blocks || detail.fill?.length) parts.push(tailTable(detail.blocks || detail.fill));
  if (change) {
    parts.unshift(`<div class="replacement">
      <h3>原子后缀替换</h3>
      <dl class="facts">${fields([["保留共同前缀", `h0–h${change.retained}`],
        ["撤销旧 G", change.old.map(block => `h${block.height}`).join("、") || "空"],
        ["以 DER 来源采用", change.new.map(block => `h${block.height}`).join("、") || "空"],
        ["D 独立末端", `h${change.d_head}`], ["下一高度 Expire", time(change.expiry_us)],
        ["新窗口 SessionEnd", time(change.session_end_us)]])}</dl>
      <details><summary>旧 G 后缀的逐高记录</summary>${tailTable(change.old)}</details>
      <details open><summary>采用的派生后缀</summary>${tailTable(change.new)}</details></div>`);
    const origin = frame.chains.g.at(-1);
    parts.push(`<details open><summary>后继职责和恢复 anchor</summary>
      <dl class="facts">${fields([...pretty(detail.duty), ["anchor native hash", origin.native],
        ["anchor 执行后 root", origin.post]])}</dl></details>`);
  }
  if (kind === "compare") parts.push('<p class="subtitle">高度差表示进度差。共高 native hash 的首处不同定位实际分歧。</p>');
  const refs = frame.refs.map(ref => `<li><code>${esc([ref.label, ref.path, ref.locator].filter(Boolean).join(" · "))}</code></li>`);
  byId("inspector").innerHTML = `<div class="inspector-head"><div><h2>${esc(frame.label)}</h2>
    <p>虚拟 τ ${time(frame.tick_us)} · ${data.mode === "sample" ? "样本 seq" : "Trace seq"} ${frame.seq}${frame.suborder ? ` · 同观察序号内第 ${frame.suborder + 1} 项` : ""}</p></div></div>
    ${parts.join("")}<details open><summary>证据位置</summary><ul class="evidence">${refs.join("")}</ul></details>`;
}

function drawTimeline() {
  byId("filters").innerHTML = lanes.map(([lane, label]) =>
    `<label><input type="checkbox" data-filter="${lane}" checked> ${label}</label>`).join("");
  const interval = end > 60000000 ? 10000000 : 5000000;
  const ticks = Array.from({length: Math.floor(end / interval) + 1}, (_, index) => index * interval);
  byId("timeline").innerHTML = `<div class="timeline-row" aria-label="刻度"><div class="lane-name">虚拟 τ</div>
    <div class="plot">${ticks.map(t => `<span class="ruler-label" style="left:${pct(t)}%">${t / 1000000}s</span>`).join("")}</div></div>`
    + lanes.map(([lane, label]) => {
      const marks = frames.flatMap((frame, index) => {
        if (frame.lane !== lane) return [];
        if (!frame.stage && ["l1_available", "signer_view"].includes(frame.kind)) return [];
        const marker = stages.find(([stage]) => stage === frame.stage);
        const caption = marker?.[1] || frame.label;
        const top = [28, 72][index % 2];
        return [`<button class="mark ${lane} ${marker ? "" : "dot"}" data-frame="${index}" title="${esc(caption)}"
          aria-label="${esc(caption)}" style="left:${pct(frame.tick_us)}%;--mark-top:${top}%">${marker ? esc(caption) : ""}</button>`];
      });
      return `<div class="timeline-row" data-lane="${lane}"><div class="lane-name">${label}</div>
        <div class="plot"><span class="track"></span><span class="cursor"></span>${marks.join("")}</div></div>`;
    }).join("");
  byId("stages").innerHTML = stages.flatMap(([stage, label]) => {
    const index = frames.findIndex(frame => frame.stage === stage);
    return index < 0 ? [] : [`<button data-stage="${stage}" data-frame="${index}">${label}</button>`];
  }).join("");
}

function compare(frame) {
  const {chains, comparison: diff} = frame;
  const max = Math.max(chains.g.length, chains.gprime.length, chains.d.length) - 1;
  const height = Math.min(fixedHeight ?? diff.first_diff ?? diff.common_height, max);
  const select = byId("compare-height");
  select.innerHTML = Array.from({length: max + 1}, (_, i) => `<option value="${i}">h${i}</option>`).join("");
  select.value = height;
  const status = byId("comparison-status");
  status.textContent = diff.first_diff == null
    ? `G 和 D 共同前缀至 h${diff.common_height}，已核对的 native hash 一致。`
    : `G 和 D 首次分歧在 h${diff.first_diff}，保留共同前缀 h0–h${diff.common_prefix}。`;
  status.classList.toggle("different", diff.first_diff != null);
  const mismatched = Boolean(chains.g[height] && chains.d[height] && chains.g[height].native !== chains.d[height].native);
  byId("chain-comparison").innerHTML = [["g", "G · 公开接纳"], ["gprime", "G′ · Inbox 认证"], ["d", "D · 独立派生"]]
    .map(([name, title]) => {
      const block = chains[name][height];
      const rows = block && [["native", block.native], ["来源", block.provenance], ["类型", kindNames[block.kind]],
        ["origin", block.origin], ["post", block.post]];
      return `<article class="chain-card ${mismatched && name !== "gprime" ? "different" : ""}">
        <h3>${title} · h${height}</h3>${rows ? `<dl class="facts">${fields(rows)}</dl>`
          : '<p class="muted empty">此链尚未物化到该高度。</p>'}</article>`;
    }).join("");
}

function seek(index) {
  cursor = Math.min(last, Math.max(0, index));
  const frame = frames[cursor];
  const {chains, duty} = frame;
  byId("clock").textContent = time(frame.tick_us);
  for (const name of ["g", "gprime", "d"]) byId(name + "-head").textContent = "h" + (chains[name].length - 1);
  byId("context").textContent = duty.tail === "NEXT" ? `R${duty.region} · β${duty.branch}/r${duty.r} [${duty.window}]` : `${duty.tail} · β${duty.branch}/r${duty.r}`;
  byId("position").textContent = position(frame.pc);
  byId("watermark").textContent = frame.watermark
    ? `q${frame.watermark.q} / ${time(frame.watermark.theta_us)}` : "—";
  byId("pending-position").textContent = frame.blocked ? `公共覆盖 WAIT · ${position(frame.blocked)}` : "";
  byId("signers").innerHTML = data.profile.members.map((address, i) => {
    const current = frame.signer_views[i];
    const rows = [["address", address], ["结论", current ? (current.eligible ? "可签" : current.reason) : "尚无观察"]];
    if (current) rows.push(...pretty(current, new Set(["refs","solver","address","eligible","signer"])));
    const observed = current && frames.findIndex(candidate => candidate.seq === current.seq && (candidate.suborder ?? 0) === current.suborder && candidate.kind === "signer_view");
    return `<article class="chain-card"><h3>Signer ${i}</h3><details ${current ? "open" : ""}>
      <summary>${esc(rows[1][1])}</summary><dl class="facts">${fields(rows)}</dl>
      ${observed >= 0 && current ? `<button data-frame="${observed}">定位观察证据</button>` : ""}</details></article>`;
  }).join("");
  byId("scrub").value = cursor;
  byId("step-label").textContent = `${cursor + 1} / ${last + 1}`;
  byId("previous").disabled = cursor === 0;
  byId("next").disabled = cursor === last;
  document.querySelectorAll(".cursor").forEach(mark => mark.style.left = pct(frame.tick_us) + "%");
  document.querySelectorAll("[data-frame]").forEach(mark => {
    const marker = Number(mark.dataset.frame);
    mark.setAttribute("aria-current", String(cursor === marker));
    if (mark.classList.contains("mark")) mark.hidden = marker > cursor;
  });
  const scroll = document.querySelector(".timeline-scroll");
  const marker = Array.from(scroll.querySelectorAll(".cursor")).find(item => !item.closest("[hidden]"));
  if (marker && scroll.clientWidth < scroll.scrollWidth) {
    const offset = marker.getBoundingClientRect().left - scroll.getBoundingClientRect().left;
    if (offset > scroll.clientWidth - 24 || offset < 115) scroll.scrollLeft += offset - scroll.clientWidth * .65;
  }
  compare(frame);
  inspect(frame);
}

function pause() {
  clearTimeout(timer);
  timer = undefined;
  byId("play").textContent = "▶ 播放";
  byId("play").setAttribute("aria-pressed", "false");
}

function playNext() {
  if (cursor === last) return pause();
  const gap = (frames[cursor + 1].tick_us - frames[cursor].tick_us) / 1800;
  timer = setTimeout(() => { seek(cursor + 1); playNext(); }, Math.max(35, Math.min(800, gap)));
}

function selectCase(index) {
  pause();
  data = datasets[index];
  frames = data.frames;
  last = frames.length - 1;
  end = Math.max(1, frames[last].tick_us);
  fixedHeight = null;
  byId("mode").classList.toggle("sample", data.mode === "sample");
  byId("mode").textContent = data.mode === "sample"
    ? "合同样本 · M5 计划值与合成记录"
    : "正式封存 M5 · " + data.provenance.candidate + " · seed " + data.profile.seed;
  byId("source").innerHTML = fields(Object.entries(data.provenance));
  drawTimeline();
  byId("scrub").max = last;
  seek(0);
}
byId("scenario").innerHTML = datasets.map((run, i) => `<option value="${i}">${esc(run.label)}</option>`).join("");
byId("scenario").addEventListener("change", event => selectCase(Number(event.target.value)));
selectCase(0);
byId("compare-height").addEventListener("change", event => { fixedHeight = Number(event.target.value); compare(frames[cursor]); });
byId("scrub").addEventListener("input", event => { pause(); seek(Number(event.target.value)); });
byId("reset").addEventListener("click", () => { pause(); fixedHeight = null; seek(0); });
byId("previous").addEventListener("click", () => { pause(); seek(cursor - 1); });
byId("next").addEventListener("click", () => { pause(); seek(cursor + 1); });
byId("play").addEventListener("click", () => {
  if (timer !== undefined) return pause();
  if (cursor === last) seek(0);
  byId("play").textContent = "Ⅱ 暂停";
  byId("play").setAttribute("aria-pressed", "true");
  playNext();
});
byId("filters").addEventListener("change", event => {
  document.querySelector(`[data-lane="${event.target.dataset.filter}"]`).hidden = !event.target.checked;
});
document.addEventListener("click", event => {
  const target = event.target.closest("[data-frame]");
  if (target) { pause(); if (target.dataset.stage) fixedHeight = null; seek(Number(target.dataset.frame)); }
});
