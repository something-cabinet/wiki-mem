/**
 * fix_skills.js — Apply corrections to wm-* skill files based on actual MCP tool schemas
 *
 * Run: node refs/fix_skills.js
 *
 * Ground truth: wm MCP server tool names + schemas from dump_tools.js output
 */

/**
 * fix_skills.js — Apply corrections to wm-* skill files based on actual MCP tool schemas
 *
 * Run: node refs/fix_skills.js
 *
 * Ground truth: wm MCP server tool names + schemas from dump_tools.js
 *
 * FIXES APPLIED:
 * 1. Tool renames: wm_search.search→wm_search.query, wm_docs.*→wm_doc.*, etc.
 * 2. Param renames: query→q (search), taskId→id (task tools), taskId→task_id (time tools)
 * 3. Remove invalid params: smart/toc/section (doc.get), appendContent (doc.update),
 *    folder/description (doc.create), appendNotes/checkAc/addAc/plan (task.update),
 *    labels→tags, description→content (task.create), category (memory.add),
 *    ref/direction/entityTypes/depth (search.resolve), doc (template.create),
 *    dryRun (template.run)
 * 4. Flags wm_code.graph calls for manual fix (no equivalent tool exists)
 */

const fs = require('fs');
const path = require('path');

const SKILLS_DIR = 'C:\\Users\\hk\\.kimaki\\projects\\vpp-rag\\wm-core\\src\\skills';
const FIXES_LOG = [];
const MANUAL_LOG = [];

// ============================================================
// TOOL NAME MAPPINGS (skill convention → actual MCP tool name)
// ============================================================

function fixFile(filePath) {
  let content = fs.readFileSync(filePath, 'utf8');
  const orig = content;
  const fileName = path.basename(path.dirname(filePath)) + '/' + path.basename(filePath);

  // --- STEP 1: Tool name renames ---
  content = content.replace(/wm_search\.search\b/g, 'wm_search.query');
  content = content.replace(/wm_docs\./g, 'wm_doc.');
  content = content.replace(/wm_tasks\./g, 'wm_task.');
  content = content.replace(/wm_templates\./g, 'wm_template.');

  // --- STEP 2: Param renames ---

  // "query": → "q": inside wm_search.query( and wm_search.retrieve( calls
  content = content.replace(/(wm_search\.query\s*\([^)]*?)"query":/g, '$1"q":');
  content = content.replace(/(wm_search\.retrieve\s*\([^)]*?)"query":/g, '$1"q":');

  // "taskId": → "id": inside wm_task.*( calls
  content = content.replace(/(wm_task\.\w+\s*\(\{[^}]*?)"taskId":/g, '$1"id":');

  // "taskId": → "task_id": inside wm_time.*( calls  
  content = content.replace(/(wm_time\.\w+\s*\(\{[^}]*?)"taskId":/g, '$1"task_id":');

  // --- STEP 2b: Remove invalid params from search.query calls ---
  // wm_search.query only accepts: q, limit, mode, offset, type. No "tag" param.
  content = content.replace(/(wm_search\.query\s*\(\{[^}]*?),[ \t\n]*"tag":\s*"[^"]*"/g, '$1');
  // Handle "tag" as first param  
  content = content.replace(/wm_search\.query\(\{\s*"tag":\s*"[^"]*"/g, 'wm_search.query({');
  // "type": "task" is not valid — change to "all"
  content = content.replace(/"type":\s*"task"/g, '"type": "all"');

  // STEP 2c is intentionally empty — template.create content param handled in manual notes

  // --- STEP 3: Remove invalid params from doc.get calls ---
  // wm_doc.get only accepts "path" — strip smart, toc, section
  content = content.replace(/(wm_doc\.get\s*\(\{)([^}]*?),\s*"smart":\s*true\s*/g, '$1$2');
  content = content.replace(/(wm_doc\.get\s*\(\{)([^}]*?),\s*"toc":\s*true\s*/g, '$1$2');
  content = content.replace(/(wm_doc\.get\s*\(\{)([^}]*?),\s*"section":\s*"[^"]*"\s*/g, '$1$2');

  // --- STEP 3b: Fix doc.list calls ---
  // wm_doc.list only accepts: folder
  content = content.replace(/wm_doc\.list\(\{\s*"tag":\s*"[^"]*"/g, 'wm_doc.list({');
  content = content.replace(/(wm_doc\.list\s*\(\{)[^}]*?,[ \t\n]*"tag":\s*"[^"]*"/g, '$1');

  // --- STEP 4: Remove invalid params from doc.update calls ---
  // wm_doc.update only accepts: path, content, tags, title
  // Remove section param (not on wm_doc.update)
  content = content.replace(/(wm_doc\.update\s*\(\{)([^}]*?),\s*"section":\s*"[^"]*"\s*/g, '$1$2');
  // Remove appendContent — flag for manual review (this is complex)
  // We can't cleanly remove appendContent that spans lines, so we flag it.
  if (/"appendContent":/g.test(content)) {
    MANUAL_LOG.push({ file: fileName, issue: 'Has "appendContent" calls that need manual restructuring (no append mechanism in wm_doc.update)' });
  }

  // --- STEP 5: Remove invalid params from doc.create calls ---
  // wm_doc.create only accepts: path (required), title (required), content, tags
  // Remove folder param (first param or subsequent)
  for (let pass = 0; pass < 3; pass++) {
    content = content.replace(/wm_doc\.create\(\{\s*"folder":\s*"[^"]*"/g, 'wm_doc.create({');
    content = content.replace(/(wm_doc\.create\s*\(\{)[^}]*?,[ \t\n]*"folder":\s*"[^"]*"/g, '$1');
  }
  // Remove description param (not valid on wm_doc.create)
  content = content.replace(/(wm_doc\.create\s*\(\{)[^}]*?,[ \t\n]*"description":\s*"[^"]*"/g, '$1');

  // --- STEP 6: Remove invalid params from task.update calls ---
  // wm_task.update only accepts: id, acceptance_criteria, assignee, content, priority, status, tags, title
  content = content.replace(/(wm_task\.update\s*\(\{[^}]*?),[ \t\n]*"appendNotes":\s*"[^"]*"/g, '$1');
  content = content.replace(/(wm_task\.update\s*\(\{[^}]*?),[ \t\n]*"checkAc":\s*\[[^\]]*\]/g, '$1');
  content = content.replace(/(wm_task\.update\s*\(\{[^}]*?),[ \t\n]*"addAc":\s*\[[^\]]*\]/g, '$1');
  content = content.replace(/(wm_task\.update\s*\(\{[^}]*?),[ \t\n]*"plan":\s*"[^"]*"/g, '$1');
  // Also catch multiline appendNotes
  content = content.replace(/"appendNotes":\s*"[^"]*"/g, '');
  // Remove "spec" param from task.update  
  content = content.replace(/(wm_task\.update\s*\(\{)([^}]*?),\s*"spec":\s*"[^"]*"\s*/g, '$1$2');

  // --- STEP 7: Fix params in task.create calls ---
  // wm_task.create accepts: id (required), title (required), acceptance_criteria, assignee, content, priority, status, tags
  // labels → tags
  content = content.replace(/(wm_task\.create\s*\(\{[^}]*?),[ \t\n]*"labels":/g, '$1, "tags":');
  // description → content
  content = content.replace(/(wm_task\.create\s*\(\{[^}]*?),[ \t\n]*"description":/g, '$1, "content":');
  // Remove "spec" from task.create
  content = content.replace(/(wm_task\.create\s*\(\{)([^}]*?),\s*"spec":\s*"[^"]*"\s*/g, '$1$2');
  // Remove "fulfills" from task.create
  content = content.replace(/(wm_task\.create\s*\(\{)([^}]*?),\s*"fulfills":\s*\[[^\]]*\]\s*/g, '$1$2');
  // Remove "order" from task.create
  content = content.replace(/(wm_task\.create\s*\(\{)([^}]*?),\s*"order":\s*\d+\s*/g, '$1$2');

  // --- STEP 8: Fix task.list calls ---
  // wm_task.list accepts: label, limit, status
  // Handle spec as first param (no leading comma)
  content = content.replace(/wm_task\.list\(\{\s*"spec":\s*"[^"]*"/g, 'wm_task.list({');
  // Handle spec as subsequent param
  content = content.replace(/(wm_task\.list\s*\(\{)[^}]*?,[ \t\n]*"spec":\s*"[^"]*"/g, '$1');

  // --- STEP 9: Remove invalid params from memory.add calls ---
  // wm_memory.add accepts: id (required), title (required), content (required), layer, tags
  // Remove "category" param
  content = content.replace(/(wm_memory\.add\s*\(\{[^}]*?),[ \t\n]*"category":\s*"[^"]*"/g, '$1');

  // --- STEP 10: Fix search.resolve calls ---
  // wm_search.resolve only accepts: q (required). Strip ref, direction, entityTypes, depth.
  // Replace the entire ref-based call with a simplified q-based call
  content = content.replace(
    /wm_search\.resolve\(\{[^}]*"ref":\s*"[^"]*"[^}]*\}\)/g,
    'wm_search.resolve({"q": "<spec-path>"})'
  );
  // Also handle already-partially-cleaned versions
  content = content.replace(/wm_search\.resolve\(\{\s*\}\)/g, 'wm_search.resolve({"q": "<spec-path>"})');

  // --- STEP 11: Fix template.create calls ---
  // wm_template.create accepts: name (required), description (required), content (required)
  content = content.replace(/(wm_template\.create\s*\(\{)([^}]*?),\s*"doc":\s*"[^"]*"\s*/g, '$1$2');

  // --- STEP 12: Fix template.run calls ---
  // wm_template.run accepts: name (required), variables (required)
  // Remove dryRun param
  content = content.replace(/(wm_template\.run\s*\(\{)([^}]*?),\s*"dryRun":\s*(true|false)\s*/g, '$1$2');

  // --- STEP 13: Fix wm_code.graph calls → wm_graph.neighbors (also removes from manual flag) ---
  // wm_code.graph doesn't exist. The closest tool is wm_graph.neighbors.
  // wm_code.graph({"query": "<page-id>"}) → wm_graph.neighbors({"id": "<page-id>"})
  // wm_code.graph({"query": "<page-id>", "neighbors": N}) → wm_graph.neighbors({"id": "<page-id>", "depth": N})
  content = content.replace(/wm_code\.graph\(\{"query":\s*"([^"]*)"\s*\}\)/g, 'wm_graph.neighbors({"id": "$1"})');
  content = content.replace(/wm_code\.graph\(\{"query":\s*"([^"]*)",\s*"neighbors":\s*(\d+)\s*\}\)/g, 'wm_graph.neighbors({"id": "$1", "depth": $2})');
  // wm_graph.neighbors also supports edge_type param — keep it if present

  // --- STEP 14: Remove old code.graph check from manual log (handled by step 13) ---

  // --- STEP 15: Clean up artifacts from removals ---
  // Remove empty params like {"path": "<x>", }
  content = content.replace(/,\s*}/g, '}');
  // Remove double commas
  content = content.replace(/,\s*,/g, ',');
  // Clean extra whitespace around empty braces
  content = content.replace(/\(\s*\{\s*,/g, '({');
  content = content.replace(/\(\{\s*\s+/g, '({');
  content = content.replace(/\s+\}/g, '}');

  // --- STEP 16: Report changes ---
  if (content !== orig) {
    const contentLines = content.split('\n');
    const origLines = orig.split('\n');
    const changes = [];
    const maxLen = Math.max(contentLines.length, origLines.length);
    for (let i = 0; i < maxLen; i++) {
      const oldL = origLines[i] || '';
      const newL = contentLines[i] || '';
      if (oldL !== newL) {
        const lineNum = i + 1;
        changes.push(`  L${lineNum}: ${oldL.trim()} → ${newL.trim()}`);
      }
    }
    FIXES_LOG.push({ file: fileName, changes });
    fs.writeFileSync(filePath, content, 'utf8');
  }
}

// Process all wm-*/SKILL.md files
const skillDirs = fs.readdirSync(SKILLS_DIR, { withFileTypes: true });
for (const dir of skillDirs) {
  if (dir.isDirectory() && dir.name.startsWith('wm-')) {
    const skillFile = path.join(SKILLS_DIR, dir.name, 'SKILL.md');
    if (fs.existsSync(skillFile)) {
      fixFile(skillFile);
    }
  }
}

// ============================================================
// REPORT
// ============================================================

console.log('=== FIX SKILLS REPORT ===');
console.log('');

if (FIXES_LOG.length > 0) {
  for (const entry of FIXES_LOG) {
    console.log(`### ${entry.file}`);
    for (const c of entry.changes) {
      console.log(c);
    }
    console.log('');
  }
}

console.log(`=== FILES MODIFIED: ${FIXES_LOG.length} ===`);
console.log('');

if (MANUAL_LOG.length > 0) {
  console.log('=== ISSUES REQUIRING MANUAL ATTENTION ===');
  console.log('');
  for (const entry of MANUAL_LOG) {
    console.log(`  ${entry.file}: ${entry.issue}`);
  }
  console.log('');

  console.log('=== DETAILED MANUAL FIX GUIDANCE ===');
  console.log('');
  console.log('1. wm_code.graph() → wm_graph.neighbors() — AUTO-FIXED (run fix_skills.js to apply).');
  console.log('   Calls replaced with wm_graph.neighbors({"id": "...", "depth": N}).');
  console.log('   Verify the auto-fix preserved intent for each call site.');
  console.log('');
  console.log('2. "appendContent" on wm_doc.update — no append mechanism exists.');
  console.log('   The "content" param replaces full content. Options:');
  console.log('   - Read existing content first, append desired text, write back via "content"');
  console.log('   - Remove and write a full replacement manually');
  console.log('   Manual review needed per call site.');
  console.log('');
  console.log('3. appendNotes/checkAc/addAc on wm_task.update — these were stripped.');
  console.log('   - checkAc → wm_task.check_ac({"id": "<id>", "criteria": [1,2]})');
  console.log('   - addAc → no tool exists. Use acceptance_criteria param on wm_task.update');
  console.log('   - appendNotes → no tool exists. Use content param if appropriate.');
  console.log('');
  console.log('4. wm_task.create missing required "id" param — needs manual addition.');
  console.log('   The actual tool requires "id". Skill files omitted it (auto-generated).');
  console.log('   Specify an explicit ID or use a convention like "<slug>-NN".');
  console.log('');
  console.log('5. wm_memory.add missing required "id" param — needs manual addition.');
  console.log('   Actual tool requires id + title + content. Skill files omitted id.');
  console.log('');
  console.log('6. wm_doc.create missing required "path" param in some calls.');
  console.log('   Actual wm_doc.create requires path + title. Some skill calls had');
  console.log('   folder+title without path. Manual review needed.');
}
