(function() {
    var selectedTier = "beginner";
    var loadedFileName = "untitled.luau";
    var diagnosticLines = {};
    var disabledRules = [];
    var allRules = [];
    var lastReport = null;

    var editor = document.getElementById("code-editor");
    var lineNumbers = document.getElementById("line-numbers");
    var highlightCode = document.getElementById("highlight-code");
    var highlightLayer = document.getElementById("highlight-layer");
    var gradeBtn = document.getElementById("grade-btn");
    var gradeText = document.getElementById("grade-text");
    var gradeSpinner = document.getElementById("grade-spinner");
    var fixBtn = document.getElementById("fix-btn");
    var fixCount = document.getElementById("fix-count");
    var tierBtns = document.querySelectorAll(".tier-btn");
    var tabs = document.querySelectorAll(".tab");
    var pastePanel = document.getElementById("paste-panel");
    var uploadPanel = document.getElementById("upload-panel");
    var fileInput = document.getElementById("file-input");
    var browseBtn = document.getElementById("browse-btn");
    var dropZone = document.getElementById("drop-zone");
    var fileNameDisplay = document.getElementById("file-name-display");
    var issueCount = document.getElementById("issue-count");
    var issueIndicator = document.getElementById("issue-indicator");
    var diagnosticsList = document.getElementById("diagnostics-list");
    var gradeBadge = document.getElementById("grade-badge");
    var settingsBtn = document.getElementById("settings-btn");
    var settingsPanel = document.getElementById("settings-panel");
    var settingsClose = document.getElementById("settings-close");
    var settingsRules = document.getElementById("settings-rules");
    var rememberToggle = document.getElementById("remember-toggle");
    var resizeHandle = document.getElementById("resize-handle");
    var codeArea = document.getElementById("code-area");
    var resultsPanel = document.getElementById("results-panel");
    var toast = document.getElementById("toast");

    var KEYWORDS = ["local","function","if","then","else","elseif","end","while","do","for","in","repeat","until","return","break","continue","and","or","not","type","export"];
    var LITERALS = ["true","false","nil"];
    var BUILTINS = ["game","workspace","script","Instance","Vector3","CFrame","Color3","UDim2","Enum","math","string","table","coroutine","task","typeof","require","print","warn","error","pcall","xpcall","pairs","ipairs","next","tostring","tonumber","setmetatable","getmetatable","select","unpack","assert","rawget","rawset"];

    function escapeHtml(s) { return s.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;"); }

    function showToast(msg) {
        toast.textContent = msg;
        toast.classList.add("visible");
        setTimeout(function(){ toast.classList.remove("visible"); }, 3000);
    }

    function tokenizeLine(line) {
        var r = "", i = 0;
        while (i < line.length) {
            if (line[i]==="-" && line[i+1]==="-") { r += '<span class="hl-comment">' + escapeHtml(line.slice(i)) + "</span>"; return r; }
            if (line[i]==='"' || line[i]==="'") {
                var q=line[i], j=i+1; while(j<line.length && line[j]!==q){if(line[j]==="\\")j++;j++;}
                r += '<span class="hl-string">' + escapeHtml(line.slice(i,j+1)) + "</span>"; i=j+1; continue;
            }
            if (line[i]===":" && /[a-zA-Z_]/.test(line[i+1]||"")) {
                var j=i+1; while(j<line.length && /[a-zA-Z0-9_]/.test(line[j]))j++;
                r += ":" + '<span class="hl-method">' + escapeHtml(line.slice(i+1,j)) + "</span>"; i=j; continue;
            }
            if (/[a-zA-Z_]/.test(line[i])) {
                var j=i; while(j<line.length && /[a-zA-Z0-9_]/.test(line[j]))j++; var w=line.slice(i,j);
                if(KEYWORDS.indexOf(w)!==-1) r+='<span class="hl-keyword">'+w+"</span>";
                else if(LITERALS.indexOf(w)!==-1) r+='<span class="hl-literal">'+w+"</span>";
                else if(BUILTINS.indexOf(w)!==-1) r+='<span class="hl-builtin">'+w+"</span>";
                else r+=escapeHtml(w); i=j; continue;
            }
            if (/[0-9]/.test(line[i])) {
                var j=i; if(line[i]==="0"&&(line[i+1]==="x"||line[i+1]==="X")){j+=2;while(j<line.length&&/[0-9a-fA-F]/.test(line[j]))j++;}
                else{while(j<line.length&&/[0-9.e]/.test(line[j]))j++;}
                r+='<span class="hl-number">'+escapeHtml(line.slice(i,j))+"</span>"; i=j; continue;
            }
            r+=escapeHtml(line[i]); i++;
        }
        return r;
    }

    function updateHighlight() {
        var lines = editor.value.split("\n"), html = "";
        for (var idx=0; idx<lines.length; idx++) {
            var ln=idx+1, cls="";
            if(diagnosticLines[ln]) cls = diagnosticLines[ln]==="Error" ? "hl-line-error" : "hl-line-warning";
            if(cls) html += '<span class="'+cls+'">' + tokenizeLine(lines[idx]) + "</span>";
            else html += tokenizeLine(lines[idx]);
            if(idx<lines.length-1) html += "\n";
        }
        highlightCode.innerHTML = html + "\n";
    }

    function updateLineNumbers() {
        var n = editor.value.split("\n").length, nums = [];
        for(var i=1;i<=n;i++) nums.push(i);
        lineNumbers.textContent = nums.join("\n");
    }

    function syncScroll() { highlightLayer.scrollTop=editor.scrollTop; highlightLayer.scrollLeft=editor.scrollLeft; lineNumbers.scrollTop=editor.scrollTop; }
    function onEditorChange() { updateLineNumbers(); updateHighlight(); }

    editor.addEventListener("input", onEditorChange);
    editor.addEventListener("scroll", syncScroll);
    editor.addEventListener("keydown", function(e) {
        if(e.key==="Tab"){e.preventDefault();var s=editor.selectionStart,end=editor.selectionEnd;editor.value=editor.value.substring(0,s)+"    "+editor.value.substring(end);editor.selectionStart=editor.selectionEnd=s+4;onEditorChange();}
    });

    tierBtns.forEach(function(btn){btn.addEventListener("click",function(){tierBtns.forEach(function(b){b.classList.remove("active");});btn.classList.add("active");selectedTier=btn.dataset.tier;});});
    tabs.forEach(function(tab){tab.addEventListener("click",function(){tabs.forEach(function(t){t.classList.remove("active");});tab.classList.add("active");if(tab.dataset.tab==="paste"){pastePanel.classList.add("active");uploadPanel.classList.remove("active");}else{pastePanel.classList.remove("active");uploadPanel.classList.add("active");}});});

    function loadFile(file) {
        if(!file)return; var reader=new FileReader();
        reader.onload=function(e){editor.value=e.target.result;loadedFileName=file.name;fileNameDisplay.textContent=file.name;onEditorChange();tabs.forEach(function(t){t.classList.remove("active");});tabs[0].classList.add("active");pastePanel.classList.add("active");uploadPanel.classList.remove("active");};
        reader.readAsText(file);
    }
    browseBtn.addEventListener("click",function(){fileInput.click();});
    fileInput.addEventListener("change",function(){loadFile(fileInput.files[0]);});
    dropZone.addEventListener("dragover",function(e){e.preventDefault();dropZone.classList.add("drag-over");});
    dropZone.addEventListener("dragleave",function(){dropZone.classList.remove("drag-over");});
    dropZone.addEventListener("drop",function(e){e.preventDefault();dropZone.classList.remove("drag-over");if(e.dataTransfer.files.length)loadFile(e.dataTransfer.files[0]);});

    function calcGrade(errors, warnings) {
        var score = Math.max(0, 100 - (errors * 15) - (warnings * 5));
        var grades = [[97,"A+"],[93,"A"],[90,"A-"],[87,"B+"],[83,"B"],[80,"B-"],[77,"C+"],[73,"C"],[70,"C-"],[67,"D+"],[63,"D"],[60,"D-"]];
        var g = "F";
        for(var i=0;i<grades.length;i++){if(score>=grades[i][0]){g=grades[i][1];break;}}
        var cls = "grade-f";
        if(g[0]==="A") cls="grade-a"; else if(g[0]==="B") cls="grade-b"; else if(g[0]==="C") cls="grade-c"; else if(g[0]==="D") cls="grade-d";
        return { grade: g, cls: cls, score: score };
    }

    function jumpToLine(line) {
        var lines = editor.value.split("\n");
        var charPos = 0;
        for(var i=0;i<line-1 && i<lines.length;i++) charPos += lines[i].length + 1;
        editor.focus();
        editor.selectionStart = editor.selectionEnd = charPos;
        var lineHeight = parseFloat(getComputedStyle(editor).lineHeight) || 20.8;
        editor.scrollTop = Math.max(0, (line - 3) * lineHeight);
        syncScroll();
        diagnosticLines["__flash"] = line;
        updateHighlight();
        setTimeout(function(){ delete diagnosticLines["__flash"]; updateHighlight(); }, 800);
    }

    function renderDiagnostics(report) {
        lastReport = report;
        diagnosticsList.innerHTML = "";
        diagnosticLines = {};
        var diags = report.diagnostics;
        var fixableCount = diags.filter(function(d){ return d.fixable; }).length;

        if(fixableCount > 0) {
            fixBtn.classList.remove("hidden");
            fixCount.classList.remove("hidden");
            fixCount.textContent = fixableCount;
        } else {
            fixBtn.classList.add("hidden");
            fixCount.classList.add("hidden");
        }

        if(!diags.length) {
            var g = calcGrade(0, 0);
            issueCount.textContent = "No issues found";
            issueIndicator.className = "clean";
            gradeBadge.textContent = g.grade;
            gradeBadge.className = g.cls;
            diagnosticsList.innerHTML = '<div id="empty-state"><p>Your code looks clean!</p></div>';
            updateHighlight(); return;
        }

        var errors=0, warnings=0;
        diags.forEach(function(d){
            if(d.severity==="Error")errors++; if(d.severity==="Warning")warnings++;
            if(d.span) diagnosticLines[d.span.line]=d.severity;
        });

        var g = calcGrade(errors, warnings);
        issueCount.textContent = diags.length + " Issue" + (diags.length!==1?"s":"") + " Found";
        issueIndicator.className = errors>0 ? "has-errors" : "has-warnings";
        gradeBadge.textContent = g.grade;
        gradeBadge.className = g.cls;

        diags.forEach(function(d) {
            var sev = d.severity.toLowerCase();
            var card = document.createElement("div");
            card.className = "diagnostic-card severity-" + sev;

            var meta = document.createElement("div"); meta.className = "diagnostic-meta";
            var chip = document.createElement("span"); chip.className = "severity-chip " + sev; chip.textContent = d.severity;
            var catChip = document.createElement("span"); catChip.className = "category-chip"; catChip.textContent = d.category || "";
            var rid = document.createElement("span"); rid.className = "rule-id"; rid.textContent = d.rule_id;
            meta.appendChild(chip); meta.appendChild(catChip); meta.appendChild(rid);
            if(d.fixable){var fc=document.createElement("span");fc.className="fixable-chip";fc.textContent="✓ fixable";meta.appendChild(fc);}
            if(d.span){var lr=document.createElement("span");lr.className="line-ref";lr.textContent="Line "+d.span.line+":"+d.span.column;meta.appendChild(lr);}

            var msg = document.createElement("div"); msg.className = "diagnostic-message"; msg.textContent = d.message;
            card.appendChild(meta); card.appendChild(msg);

            if(d.suggestion){var fix=document.createElement("div");fix.className="diagnostic-fix";var fl=document.createElement("span");fl.className="fix-label";fl.textContent="Fix:";var fcode=document.createElement("code");fcode.className="fix-code";fcode.textContent=d.suggestion;fix.appendChild(fl);fix.appendChild(fcode);card.appendChild(fix);}

            if(d.span) { card.addEventListener("click", (function(line){return function(){jumpToLine(line);};})(d.span.line)); }
            diagnosticsList.appendChild(card);
        });
        updateHighlight();
    }

    function setGrading(on) {
        if(on) { gradeBtn.classList.add("grading"); gradeText.textContent=""; gradeSpinner.classList.remove("hidden"); }
        else { gradeBtn.classList.remove("grading"); gradeText.textContent="GRADE"; gradeSpinner.classList.add("hidden"); }
    }

    gradeBtn.addEventListener("click", function() {
        var source = editor.value.trim();
        if(!source) return;
        setGrading(true);
        fixBtn.classList.add("hidden");
        diagnosticLines = {};
        issueCount.textContent = "Grading...";
        issueIndicator.className = "";
        gradeBadge.className = "hidden";
        diagnosticsList.innerHTML = '<div id="empty-state"><div class="spinner" style="width:24px;height:24px;border-width:3px;"></div></div>';
        updateHighlight();

        var request = { source: source, tier: selectedTier, file_name: loadedFileName, disabled_rules: disabledRules };
        if(window.__TAURI__) {
            window.__TAURI__.core.invoke("grade_luau", { request: request })
                .then(function(report){ renderDiagnostics(report); })
                .catch(function(err){
                    issueCount.textContent="Syntax Error";
                    issueIndicator.className="has-errors";
                    gradeBadge.className="hidden";
                    var errMsg = String(err);
                    if(errMsg.indexOf("parse") !== -1 || errMsg.indexOf("Parse") !== -1) {
                        errMsg = "Your code has syntax errors and could not be parsed. Fix the syntax and try again.\n\n" + errMsg;
                    }
                    diagnosticsList.innerHTML='<div id="empty-state" style="flex-direction:column;gap:8px;text-align:center;"><p style="color:#f38ba8;font-weight:600;">Could not grade this code</p><p style="color:#6c7086;font-size:12px;max-width:500px;">'+escapeHtml(errMsg)+'</p></div>';
                })
                .finally(function(){ setGrading(false); });
        }
    });

    fixBtn.addEventListener("click", function() {
        var source = editor.value.trim();
        if(!source) return;
        var request = { source: source, tier: selectedTier, file_name: loadedFileName, disabled_rules: disabledRules };
        if(window.__TAURI__) {
            window.__TAURI__.core.invoke("apply_fixes", { request: request })
                .then(function(report) {
                    if(report.applied.length > 0) {
                        editor.value = report.fixed_source;
                        onEditorChange();
                        showToast("Applied " + report.applied.length + " fix" + (report.applied.length !== 1 ? "es" : ""));
                        setTimeout(function(){ gradeBtn.click(); }, 300);
                    } else {
                        showToast("No fixes available");
                    }
                })
                .catch(function(err){ showToast("Fix error: " + String(err)); });
        }
    });

    settingsBtn.addEventListener("click",function(){settingsPanel.classList.toggle("hidden");});
    settingsClose.addEventListener("click",function(){settingsPanel.classList.add("hidden");});

    function loadSettings() {
        try { var saved = localStorage.getItem("luau-grader-disabled"); if(saved) { disabledRules = JSON.parse(saved); rememberToggle.checked = true; } } catch(e){}
    }
    function saveSettings() {
        if(rememberToggle.checked) localStorage.setItem("luau-grader-disabled", JSON.stringify(disabledRules));
        else localStorage.removeItem("luau-grader-disabled");
    }
    rememberToggle.addEventListener("change", saveSettings);

    function buildSettingsPanel(rules) {
        allRules = rules;
        settingsRules.innerHTML = "";
        var categories = {};
        rules.forEach(function(r){
            if(!categories[r.category]) categories[r.category] = [];
            categories[r.category].push(r);
        });
        Object.keys(categories).sort().forEach(function(cat) {
            var header = document.createElement("div"); header.className = "settings-category"; header.textContent = cat;
            settingsRules.appendChild(header);
            categories[cat].forEach(function(rule) {
                var row = document.createElement("label"); row.className = "settings-rule";
                var cb = document.createElement("input"); cb.type = "checkbox"; cb.checked = disabledRules.indexOf(rule.id) === -1;
                cb.addEventListener("change", function() {
                    if(cb.checked) disabledRules = disabledRules.filter(function(id){return id !== rule.id;});
                    else disabledRules.push(rule.id);
                    saveSettings();
                });
                var idSpan = document.createElement("span"); idSpan.className = "settings-rule-id"; idSpan.textContent = rule.id;
                var descSpan = document.createElement("span"); descSpan.className = "settings-rule-desc"; descSpan.textContent = rule.description;
                if(rule.fixable) descSpan.textContent += " ✓";
                row.appendChild(cb); row.appendChild(idSpan); row.appendChild(descSpan);
                settingsRules.appendChild(row);
            });
        });
    }

    if(window.__TAURI__) {
        window.__TAURI__.core.invoke("list_rules").then(buildSettingsPanel).catch(function(){});
    }

    var isResizing = false;
    resizeHandle.addEventListener("mousedown", function(e) {
        e.preventDefault(); isResizing = true; resizeHandle.classList.add("active");
        document.body.style.cursor = "ns-resize"; document.body.style.userSelect = "none";
    });
    document.addEventListener("mousemove", function(e) {
        if(!isResizing) return;
        var workspaceRect = document.getElementById("workspace").getBoundingClientRect();
        var handleHeight = 5;
        var totalHeight = workspaceRect.height - handleHeight;
        var topHeight = e.clientY - workspaceRect.top - 52;
        topHeight = Math.max(120, Math.min(totalHeight - 100, topHeight));
        var bottomHeight = totalHeight - topHeight;
        codeArea.style.flex = "none"; codeArea.style.height = topHeight + "px";
        resultsPanel.style.height = bottomHeight + "px";
    });
    document.addEventListener("mouseup", function() {
        if(isResizing) { isResizing = false; resizeHandle.classList.remove("active"); document.body.style.cursor = ""; document.body.style.userSelect = ""; }
    });

    loadSettings();
    onEditorChange();
})();