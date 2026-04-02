(function() {
    var selectedTier = "beginner";
    var loadedFileName = "untitled.luau";
    var diagnosticLines = {};
    var disabledRules = [];
    var allRules = [];
    var lastReport = null;
    var pendingFixSource = null;
    var heatmapData = {};

    var editor = document.getElementById("code-editor");
    var lineNumbers = document.getElementById("line-numbers");
    var highlightCode = document.getElementById("highlight-code");
    var highlightLayer = document.getElementById("highlight-layer");
    var gradeBtn = document.getElementById("grade-btn");
    var gradeText = document.getElementById("grade-text");
    var gradeSpinner = document.getElementById("grade-spinner");
    var fixBtn = document.getElementById("fix-btn");
    var fixCount = document.getElementById("fix-count");
    var exportBtn = document.getElementById("export-btn");
    var tierBtns = document.querySelectorAll(".tier-btn");
    var tabs = document.querySelectorAll(".tab");
    var pastePanel = document.getElementById("paste-panel");
    var uploadPanel = document.getElementById("upload-panel");
    var workspacePanel = document.getElementById("workspace-panel");
    var browseWorkspaceBtn = document.getElementById("browse-workspace-btn");
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
    var gradeDashboard = document.getElementById("grade-dashboard");
    var chartToggleBtn = document.getElementById("chart-toggle-btn");
    var radarContainer = document.getElementById("radar-chart-container");
    var dimensionBars = document.getElementById("dimension-bars");
    var diffOverlay = document.getElementById("diff-overlay");
    var diffCancelBtn = document.getElementById("diff-cancel-btn");
    var diffApplyBtn = document.getElementById("diff-apply-btn");

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
        var lines = editor.value.split("\n");
        var n = lines.length;
        var hasHeatmap = Object.keys(heatmapData).length > 0;
        if (!hasHeatmap) {
            var nums = [];
            for(var i=1;i<=n;i++) nums.push(i);
            lineNumbers.textContent = nums.join("\n");
            return;
        }
        lineNumbers.innerHTML = "";
        for(var i=1;i<=n;i++) {
            var span = document.createElement("span");
            span.textContent = i;
            var density = heatmapData[i] || 0;
            if (density >= 3) span.className = "line-num-high";
            else if (density >= 2) span.className = "line-num-med";
            else if (density >= 1) span.className = "line-num-low";
            else span.className = "line-num-clean";
            lineNumbers.appendChild(span);
            if (i < n) lineNumbers.appendChild(document.createTextNode("\n"));
        }
    }

    function syncScroll() { highlightLayer.scrollTop=editor.scrollTop; highlightLayer.scrollLeft=editor.scrollLeft; lineNumbers.scrollTop=editor.scrollTop; }
    function onEditorChange() { updateLineNumbers(); updateHighlight(); }

    editor.addEventListener("input", onEditorChange);
    editor.addEventListener("scroll", syncScroll);
    editor.addEventListener("keydown", function(e) {
        if(e.key==="Tab"){e.preventDefault();var s=editor.selectionStart,end=editor.selectionEnd;editor.value=editor.value.substring(0,s)+"    "+editor.value.substring(end);editor.selectionStart=editor.selectionEnd=s+4;onEditorChange();}
    });

    tierBtns.forEach(function(btn){btn.addEventListener("click",function(){tierBtns.forEach(function(b){b.classList.remove("active");});btn.classList.add("active");selectedTier=btn.dataset.tier;});});
    tabs.forEach(function(tab) {
        tab.addEventListener("click", function() {
            tabs.forEach(function(t) { t.classList.remove("active"); });
            tab.classList.add("active");
            pastePanel.classList.remove("active");
            uploadPanel.classList.remove("active");
            workspacePanel.classList.remove("active");
            
            if (tab.dataset.tab === "paste") {
                pastePanel.classList.add("active");
                gradeBtn.style.display = "flex";
                resultsPanel.style.display = "flex";
                resizeHandle.style.display = "block";
            } else if (tab.dataset.tab === "upload") {
                uploadPanel.classList.add("active");
                gradeBtn.style.display = "flex";
                resultsPanel.style.display = "flex";
                resizeHandle.style.display = "block";
            } else if (tab.dataset.tab === "workspace") {
                workspacePanel.classList.add("active");
                gradeBtn.style.display = "none";
                resultsPanel.style.display = "none";
                resizeHandle.style.display = "none";
                fixBtn.classList.add("hidden");
                exportBtn.classList.add("hidden");
                gradeBadge.classList.add("hidden");
            }
        });
    });

    var wsContent = document.getElementById("workspace-content");
    var wsContainer = document.getElementById("workspace-diagnostics-container");
    var wsList = document.getElementById("workspace-diagnostics-list");
    var wsIssueCount = document.getElementById("ws-issue-count");
    var wsIssueIndicator = document.getElementById("ws-issue-indicator");
    var repickBtn = document.getElementById("repick-workspace-btn");

    function pickWorkspace() {
        if (!window.__TAURI__) {
            showToast("Cannot open native dialog outside of app.");
            return;
        }
        window.__TAURI__.core.invoke("pick_workspace_folder").then(function(folderPath) {
            if (folderPath) {
                fileNameDisplay.textContent = "Workspace: " + folderPath.split(/[\\/]/).pop();
                wsContent.classList.add("hidden");
                wsContainer.classList.remove("hidden");
                wsIssueCount.textContent = "Analyzing Workspace...";
                wsIssueIndicator.style.backgroundColor = "#a6adc8";
                wsList.innerHTML = "<div style='padding: 20px; color: #a6adc8; display: flex; align-items: center; justify-content: center; height: 100%;'>Scanning all luau files...</div>";
                
                window.__TAURI__.core.invoke("analyze_workspace", { path: folderPath }).then(function(diags) {
                    wsList.innerHTML = "";
                    
                    if (!diags || diags.length === 0) {
                        wsIssueCount.textContent = "0 Workspace Issues";
                        wsIssueIndicator.style.backgroundColor = "#a6e3a1";
                        var empty = document.createElement("div");
                        empty.className = "empty-state";
                        empty.innerHTML = "<div style='text-align:center;'><h3>Perfect!</h3><p>Your workspace is free of cyclic dependencies and dead code.</p></div>";
                        wsList.appendChild(empty);
                        return;
                    }
                    
                    var errs = 0; var warns = 0;
                    var frag = document.createDocumentFragment();
                    var maxRender = 500;
                    var rendered = 0;
                    
                    diags.forEach(function(d) {
                        if (d.severity === "Error") errs++;
                        else if (d.severity === "Warning") warns++;
                        else if (d.severity === "Info") warns++;
                        
                        if (rendered < maxRender) {
                            var li = document.createElement("div");
                            li.className = "diagnostic-card severity-" + d.severity.toLowerCase();
                            var meta = document.createElement("div");
                            meta.className = "diagnostic-meta";
                            meta.innerHTML = "<span class='severity-chip " + d.severity.toLowerCase() + "'>" + escapeHtml(d.severity) + "</span>" +
                                             "<span class='rule-id'>" + escapeHtml(d.rule_id) + "</span>";
                            
                            if (d.file_path) {
                                var lr = document.createElement("span");
                                lr.className = "line-ref";
                                lr.textContent = escapeHtml(d.file_path.split(/[\\/]/).pop());
                                meta.appendChild(lr);
                            }
                            
                            var msg = document.createElement("div");
                            msg.className = "diagnostic-message";
                            msg.innerHTML = escapeHtml(d.message);
                            
                            li.appendChild(meta);
                            li.appendChild(msg);
                            frag.appendChild(li);
                            rendered++;
                        }
                    });
                    
                    if (diags.length > maxRender) {
                        var notice = document.createElement("div");
                        notice.className = "diagnostic-card severity-info";
                        notice.innerHTML = "<i>Showing first " + maxRender + " issues to ensure UI performance in large scale codebases.</i>";
                        frag.appendChild(notice);
                    }
                    
                    wsList.appendChild(frag);
                    wsIssueCount.textContent = errs + " Errors, " + warns + " Warnings";
                    wsIssueIndicator.style.backgroundColor = errs > 0 ? "#f38ba8" : (warns > 0 ? "#f9e2af" : "#a6e3a1");
                }).catch(function(err) {
                    wsIssueCount.textContent = "Analysis Failed";
                    wsIssueIndicator.style.backgroundColor = "#f38ba8";
                    showToast("Error analyzing workspace: " + err);
                });
            }
        });
    }

    if (browseWorkspaceBtn) {
        browseWorkspaceBtn.addEventListener("click", pickWorkspace);
    }
    if (repickBtn) {
        repickBtn.addEventListener("click", pickWorkspace);
    }

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

    function gradeColor(g) {
        if (!g) return "grade-f";
        var c = g.charAt(0);
        if (c==="A") return "grade-a";
        if (c==="B") return "grade-b";
        if (c==="C") return "grade-c";
        if (c==="D") return "grade-d";
        return "grade-f";
    }

    function dimColor(score) {
        if (score >= 80) return "#a6e3a1";
        if (score >= 60) return "#f9e2af";
        return "#f38ba8";
    }

    function animateScoreRing(score, grade) {
        var ringFill = document.getElementById("score-ring-fill");
        var ringGrade = document.getElementById("score-ring-grade");
        var ringScore = document.getElementById("score-ring-score");
        var circumference = 2 * Math.PI * 52;
        var offset = circumference - (score / 100) * circumference;

        ringFill.style.transition = "none";
        ringFill.setAttribute("stroke-dashoffset", circumference);
        ringGrade.textContent = "--";
        ringScore.textContent = "0";

        var color = dimColor(score);
        ringFill.setAttribute("stroke", color);

        requestAnimationFrame(function() {
            requestAnimationFrame(function() {
                ringFill.style.transition = "stroke-dashoffset 1.2s ease-out";
                ringFill.setAttribute("stroke-dashoffset", offset);
            });
        });

        var counter = { val: 0 };
        var duration = 1200;
        var startTime = performance.now();
        function tick(now) {
            var elapsed = now - startTime;
            var progress = Math.min(elapsed / duration, 1);
            var eased = 1 - Math.pow(1 - progress, 3);
            var current = Math.round(eased * score);
            ringScore.textContent = current;
            if (progress < 1) requestAnimationFrame(tick);
            else ringGrade.textContent = grade;
        }
        requestAnimationFrame(tick);
    }

    function renderDimensionBars(dimensions) {
        var container = document.getElementById("dimension-bars");
        container.innerHTML = "";
        dimensions.forEach(function(dim, idx) {
            var row = document.createElement("div");
            row.className = "dim-row";

            var label = document.createElement("span");
            label.className = "dim-label";
            label.textContent = dim.name;

            var track = document.createElement("div");
            track.className = "dim-track";

            var fill = document.createElement("div");
            fill.className = "dim-fill";
            fill.style.width = "0%";
            fill.style.backgroundColor = dimColor(dim.score);

            var scoreLabel = document.createElement("span");
            scoreLabel.className = "dim-score";
            scoreLabel.textContent = dim.score;

            track.appendChild(fill);
            row.appendChild(label);
            row.appendChild(track);
            row.appendChild(scoreLabel);
            container.appendChild(row);

            setTimeout(function() {
                fill.style.width = dim.score + "%";
            }, 100 + idx * 80);
        });
    }

    function renderRadarChart(dimensions) {
        var svg = document.getElementById("radar-chart");
        svg.innerHTML = "";
        var cx = 150, cy = 150, maxR = 110;
        var n = dimensions.length;
        var angleStep = (2 * Math.PI) / n;
        var startAngle = -Math.PI / 2;
        var levels = [20, 40, 60, 80, 100];

        levels.forEach(function(level) {
            var r = (level / 100) * maxR;
            var pts = [];
            for (var i = 0; i < n; i++) {
                var angle = startAngle + i * angleStep;
                pts.push((cx + r * Math.cos(angle)).toFixed(1) + "," + (cy + r * Math.sin(angle)).toFixed(1));
            }
            var poly = document.createElementNS("http://www.w3.org/2000/svg", "polygon");
            poly.setAttribute("points", pts.join(" "));
            poly.setAttribute("class", "radar-grid");
            svg.appendChild(poly);
        });

        for (var i = 0; i < n; i++) {
            var angle = startAngle + i * angleStep;
            var line = document.createElementNS("http://www.w3.org/2000/svg", "line");
            line.setAttribute("x1", cx);
            line.setAttribute("y1", cy);
            line.setAttribute("x2", (cx + maxR * Math.cos(angle)).toFixed(1));
            line.setAttribute("y2", (cy + maxR * Math.sin(angle)).toFixed(1));
            line.setAttribute("class", "radar-axis");
            svg.appendChild(line);
        }

        var zeroPoints = [];
        var dataPoints = [];
        for (var i = 0; i < n; i++) {
            zeroPoints.push(cx + "," + cy);
            var angle = startAngle + i * angleStep;
            var r = (dimensions[i].score / 100) * maxR;
            dataPoints.push({
                x: cx + r * Math.cos(angle),
                y: cy + r * Math.sin(angle),
                score: dimensions[i].score,
                name: dimensions[i].name,
                angle: angle
            });
        }

        var polygon = document.createElementNS("http://www.w3.org/2000/svg", "polygon");
        polygon.setAttribute("points", zeroPoints.join(" "));
        polygon.setAttribute("class", "radar-polygon");
        svg.appendChild(polygon);

        setTimeout(function() {
            var finalPts = dataPoints.map(function(p) {
                return p.x.toFixed(1) + "," + p.y.toFixed(1);
            });
            polygon.setAttribute("points", finalPts.join(" "));
            polygon.classList.add("animated");
        }, 50);

        dataPoints.forEach(function(p) {
            var labelR = maxR + 22;
            var lx = cx + labelR * Math.cos(p.angle);
            var ly = cy + labelR * Math.sin(p.angle);

            var g = document.createElementNS("http://www.w3.org/2000/svg", "g");
            g.setAttribute("class", "radar-dot-group");

            var dot = document.createElementNS("http://www.w3.org/2000/svg", "circle");
            dot.setAttribute("cx", p.x.toFixed(1));
            dot.setAttribute("cy", p.y.toFixed(1));
            dot.setAttribute("r", "3");
            dot.setAttribute("class", "radar-dot");
            g.appendChild(dot);

            var shortName = p.name;
            if (shortName.length > 12) shortName = shortName.substring(0, 11) + ".";
            var label = document.createElementNS("http://www.w3.org/2000/svg", "text");
            label.setAttribute("x", lx.toFixed(1));
            label.setAttribute("y", (ly - 4).toFixed(1));
            label.setAttribute("class", "radar-label");
            label.textContent = shortName;
            g.appendChild(label);

            var val = document.createElementNS("http://www.w3.org/2000/svg", "text");
            val.setAttribute("x", lx.toFixed(1));
            val.setAttribute("y", (ly + 9).toFixed(1));
            val.setAttribute("class", "radar-value");
            val.textContent = p.score;
            g.appendChild(val);

            svg.appendChild(g);
        });
    }

    function renderFunctionGrades(grades) {
        var section = document.getElementById("function-grades-section");
        var list = document.getElementById("function-grades-list");
        list.innerHTML = "";

        if (!grades || grades.length === 0) {
            section.classList.add("hidden");
            return;
        }

        section.classList.remove("hidden");
        grades.forEach(function(fg) {
            var row = document.createElement("div");
            row.className = "func-grade-row";

            var name = document.createElement("span");
            name.className = "func-name";
            name.textContent = fg.name;

            var line = document.createElement("span");
            line.className = "func-line";
            line.textContent = "line " + fg.line;

            var grade = document.createElement("span");
            grade.className = "func-grade " + gradeColor(fg.grade);
            grade.textContent = fg.grade;

            var stats = document.createElement("span");
            stats.className = "func-stats";
            stats.textContent = "complexity:" + fg.complexity + "  lines:" + fg.lines;

            row.appendChild(name);
            row.appendChild(line);
            row.appendChild(grade);
            row.appendChild(stats);
            list.appendChild(row);

            if (fg.line > 0) {
                row.style.cursor = "pointer";
                row.addEventListener("click", function() { jumpToLine(fg.line); });
            }
        });
    }

    function renderDebt(debt) {
        var section = document.getElementById("debt-section");
        var total = document.getElementById("debt-total");
        var breakdown = document.getElementById("debt-breakdown");

        if (!debt || debt.total_minutes === 0) {
            section.classList.add("hidden");
            return;
        }

        section.classList.remove("hidden");
        total.textContent = debt.total_minutes + " minutes estimated";
        breakdown.innerHTML = "";

        debt.breakdown.forEach(function(item) {
            if (item.minutes === 0) return;
            var row = document.createElement("div");
            row.className = "debt-row";
            row.innerHTML = '<span class="debt-cat">' + escapeHtml(item.category) + '</span>' +
                '<span class="debt-min">' + item.minutes + ' min</span>' +
                '<span class="debt-count">' + item.count + ' issues</span>';
            breakdown.appendChild(row);
        });
    }

    function renderImprovement(improvement) {
        var section = document.getElementById("improvement-section");
        var header = document.getElementById("improvement-header");
        var steps = document.getElementById("improvement-steps");

        if (!improvement || !improvement.fixes_needed || improvement.fixes_needed.length === 0) {
            section.classList.add("hidden");
            return;
        }

        section.classList.remove("hidden");
        header.innerHTML = '<span class="imp-current">' + improvement.current_grade + '</span>' +
            '<span class="imp-arrow">&#8594;</span>' +
            '<span class="imp-projected">' + improvement.projected_grade + '</span>';

        steps.innerHTML = "";
        improvement.fixes_needed.forEach(function(fix, i) {
            var step = document.createElement("div");
            step.className = "imp-step";
            step.innerHTML = '<span class="imp-num">' + (i + 1) + '</span>' +
                '<span class="imp-desc">' + escapeHtml(fix.description) + '</span>' +
                '<span class="imp-impact">+' + Math.round(fix.score_impact) + ' pts</span>' +
                '<span class="imp-effort">~' + fix.effort_minutes + ' min</span>';
            steps.appendChild(step);
        });
    }

    function renderStrengths(strengths) {
        var section = document.getElementById("strengths-section");
        var list = document.getElementById("strengths-list");

        if (!strengths || strengths.length === 0) {
            section.classList.add("hidden");
            return;
        }

        section.classList.remove("hidden");
        list.innerHTML = "";
        strengths.forEach(function(s) {
            var item = document.createElement("div");
            item.className = "strength-item";
            item.innerHTML = '<span class="strength-icon">+</span>' + escapeHtml(s);
            list.appendChild(item);
        });
    }

    function renderPatterns(patterns) {
        var section = document.getElementById("patterns-section");
        var list = document.getElementById("patterns-list");

        if (!patterns || patterns.length === 0) {
            section.classList.add("hidden");
            return;
        }

        section.classList.remove("hidden");
        list.innerHTML = "";
        patterns.forEach(function(p) {
            var tag = document.createElement("span");
            tag.className = "pattern-tag";
            tag.textContent = p;
            list.appendChild(tag);
        });
    }

    function buildHeatmapData(diags) {
        heatmapData = {};
        diags.forEach(function(d) {
            if (d.span) {
                var line = d.span.line;
                if (!heatmapData[line]) heatmapData[line] = 0;
                heatmapData[line]++;
                if (line > 1) {
                    if (!heatmapData[line - 1]) heatmapData[line - 1] = 0;
                }
                if (!heatmapData[line + 1]) heatmapData[line + 1] = 0;
            }
        });
    }

    function renderGradeDashboard(report) {
        gradeDashboard.classList.remove("hidden");
        exportBtn.classList.remove("hidden");

        buildHeatmapData(report.diagnostics);
        updateLineNumbers();

        animateScoreRing(Math.round(report.overall_score), report.grade);
        renderDimensionBars(report.dimensions);
        if (!radarContainer.classList.contains("hidden")) {
            renderRadarChart(report.dimensions);
        }
        renderFunctionGrades(report.function_grades);
        renderDebt(report.debt);
        renderImprovement(report.improvement);
        renderStrengths(report.strengths);
        renderPatterns(report.detected_patterns);
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

        var errors=0, warnings=0, infos=0;
        diags.forEach(function(d){
            if(d.severity==="Error")errors++; 
            else if(d.severity==="Warning")warnings++;
            else if(d.severity==="Info")infos++;
            if(d.span) diagnosticLines[d.span.line]=d.severity;
        });

        var realIssues = errors + warnings;
        var breakdown = [];
        if (errors > 0) breakdown.push(errors + (errors === 1 ? " Error" : " Errors"));
        if (warnings > 0) breakdown.push(warnings + (warnings === 1 ? " Warning" : " Warnings"));
        if (infos > 0) breakdown.push(infos + (infos === 1 ? " Suggestion" : " Suggestions"));

        var summaryText = "";
        if (realIssues === 0) {
            if (infos === 0) {
                summaryText = "No issues found";
            } else {
                summaryText = breakdown.join(", ");
            }
        } else {
            summaryText = realIssues + " Issue" + (realIssues !== 1 ? "s" : "") + " Found (" + breakdown.join(", ") + ")";
        }

        if(!diags.length) {
            issueCount.textContent = "No issues found";
            issueIndicator.className = "clean";
            gradeBadge.textContent = report.grade || "A+";
            gradeBadge.className = gradeColor(report.grade || "A+");
            diagnosticsList.innerHTML = '<div id="empty-state"><p>Your code looks clean!</p></div>';
            updateHighlight(); return;
        }

        issueCount.textContent = summaryText;
        issueIndicator.className = errors>0 ? "has-errors" : "has-warnings";
        gradeBadge.textContent = report.grade || "--";
        gradeBadge.className = gradeColor(report.grade);

        diags.forEach(function(d) {
            var sev = d.severity.toLowerCase();
            var card = document.createElement("div");
            card.className = "diagnostic-card severity-" + sev;

            var meta = document.createElement("div"); meta.className = "diagnostic-meta";
            var chip = document.createElement("span"); chip.className = "severity-chip " + sev; chip.textContent = d.severity;
            var catChip = document.createElement("span"); catChip.className = "category-chip"; catChip.textContent = d.category || "";
            var rid = document.createElement("span"); rid.className = "rule-id"; rid.textContent = d.rule_id;
            meta.appendChild(chip); meta.appendChild(catChip); meta.appendChild(rid);
            if(d.fixable){var fc=document.createElement("span");fc.className="fixable-chip";fc.textContent="fixable";meta.appendChild(fc);}
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
        exportBtn.classList.add("hidden");
        heatmapData = {};
        diagnosticLines = {};
        issueCount.textContent = "Grading...";
        issueIndicator.className = "";
        gradeBadge.className = "hidden";
        gradeDashboard.classList.add("hidden");
        diagnosticsList.innerHTML = '<div id="empty-state"><div class="spinner" style="width:24px;height:24px;border-width:3px;"></div></div>';
        updateHighlight();

        var request = { source: source, tier: selectedTier, file_name: loadedFileName, disabled_rules: disabledRules };
        if(window.__TAURI__) {
            window.__TAURI__.core.invoke("grade_luau", { request: request })
                .then(function(report){
                    renderGradeDashboard(report);
                    renderDiagnostics(report);
                })
                .catch(function(err){
                    issueCount.textContent="Syntax Error";
                    issueIndicator.className="has-errors";
                    gradeBadge.className="hidden";
                    gradeDashboard.classList.add("hidden");
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
                        pendingFixSource = report.fixed_source;
                        showDiffModal(source, report.fixed_source, report.applied);
                    } else {
                        showToast("No fixes available");
                    }
                })
                .catch(function(err){ showToast("Fix error: " + String(err)); });
        }
    });

    chartToggleBtn.addEventListener("click", function() {
        var showing = !radarContainer.classList.contains("hidden");
        if (showing) {
            radarContainer.classList.add("hidden");
            dimensionBars.style.display = "";
            chartToggleBtn.textContent = "Show Chart";
            chartToggleBtn.classList.remove("active");
        } else {
            radarContainer.classList.remove("hidden");
            dimensionBars.style.display = "none";
            chartToggleBtn.textContent = "Show Bars";
            chartToggleBtn.classList.add("active");
            if (lastReport && lastReport.dimensions) {
                renderRadarChart(lastReport.dimensions);
            }
        }
    });

    exportBtn.addEventListener("click", function() {
        var source = editor.value.trim();
        if(!source) return;
        var request = { source: source, tier: selectedTier, file_name: loadedFileName, disabled_rules: disabledRules };
        if(window.__TAURI__) {
            window.__TAURI__.core.invoke("export_report", { request: request })
                .then(function(markdown) {
                    var blob = new Blob([markdown], { type: "text/markdown" });
                    var url = URL.createObjectURL(blob);
                    var a = document.createElement("a");
                    a.href = url;
                    a.download = loadedFileName.replace(/\.(luau|lua)$/i, "") + "_report.md";
                    document.body.appendChild(a);
                    a.click();
                    document.body.removeChild(a);
                    URL.revokeObjectURL(url);
                    showToast("Report exported");
                })
                .catch(function(err){ showToast("Export error: " + String(err)); });
        }
    });

    function showDiffModal(before, after, applied) {
        var beforeLines = before.split("\n");
        var afterLines = after.split("\n");
        var maxLen = Math.max(beforeLines.length, afterLines.length);

        var changedBefore = {};
        var changedAfter = {};
        var addCount = 0;
        var removeCount = 0;

        for (var i = 0; i < maxLen; i++) {
            var bLine = i < beforeLines.length ? beforeLines[i] : undefined;
            var aLine = i < afterLines.length ? afterLines[i] : undefined;
            if (bLine !== aLine) {
                if (bLine !== undefined) { changedBefore[i] = true; removeCount++; }
                if (aLine !== undefined) { changedAfter[i] = true; addCount++; }
            }
        }

        var stats = document.getElementById("diff-stats");
        stats.innerHTML = '<span class="diff-stat-add">+' + addCount + ' added</span>' +
            '<span class="diff-stat-remove">' + removeCount + ' removed</span>' +
            '<span>' + applied.length + ' fix' + (applied.length !== 1 ? 'es' : '') + '</span>';

        var beforeHtml = "";
        beforeLines.forEach(function(line, idx) {
            var cls = changedBefore[idx] ? "diff-line-removed" : "diff-line-context";
            beforeHtml += '<span class="' + cls + '">' + escapeHtml(line) + '</span>\n';
        });

        var afterHtml = "";
        afterLines.forEach(function(line, idx) {
            var cls = changedAfter[idx] ? "diff-line-added" : "diff-line-context";
            afterHtml += '<span class="' + cls + '">' + escapeHtml(line) + '</span>\n';
        });

        document.getElementById("diff-content-before").innerHTML = beforeHtml;
        document.getElementById("diff-content-after").innerHTML = afterHtml;
        diffOverlay.classList.remove("hidden");
    }

    diffCancelBtn.addEventListener("click", function() {
        diffOverlay.classList.add("hidden");
        pendingFixSource = null;
    });

    diffApplyBtn.addEventListener("click", function() {
        if (pendingFixSource) {
            editor.value = pendingFixSource;
            onEditorChange();
            diffOverlay.classList.add("hidden");
            showToast("Changes applied");
            pendingFixSource = null;
            setTimeout(function(){ gradeBtn.click(); }, 300);
        }
    });

    diffOverlay.addEventListener("click", function(e) {
        if (e.target === diffOverlay) {
            diffOverlay.classList.add("hidden");
            pendingFixSource = null;
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
                if(rule.fixable) descSpan.textContent += " [fixable]";
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

    var bgUploadBtn = document.getElementById("bg-upload-btn");
    var bgClearBtn = document.getElementById("bg-clear-btn");
    var bgFileInput = document.getElementById("bg-file-input");
    var bgPreview = document.getElementById("bg-preview");
    var bgNoImage = document.getElementById("bg-no-image");
    var bgOpacitySlider = document.getElementById("bg-opacity-slider");
    var bgOpacityValue = document.getElementById("bg-opacity-value");

    var DB_NAME = "luau-grader-bg-db";
    var DB_STORE = "backgrounds";
    var DB_KEY = "user-bg";
    var MAX_FILE_SIZE = 25 * 1024 * 1024;

    function openBgDb(callback) {
        var request = indexedDB.open(DB_NAME, 1);
        request.onupgradeneeded = function(e) {
            var db = e.target.result;
            if (!db.objectStoreNames.contains(DB_STORE)) {
                db.createObjectStore(DB_STORE);
            }
        };
        request.onsuccess = function(e) { callback(e.target.result); };
        request.onerror = function() { callback(null); };
    }

    function saveBgToDb(dataUrl) {
        openBgDb(function(db) {
            if (!db) return;
            var tx = db.transaction(DB_STORE, "readwrite");
            tx.objectStore(DB_STORE).put(dataUrl, DB_KEY);
        });
    }

    function loadBgFromDb(callback) {
        openBgDb(function(db) {
            if (!db) { callback(null); return; }
            var tx = db.transaction(DB_STORE, "readonly");
            var req = tx.objectStore(DB_STORE).get(DB_KEY);
            req.onsuccess = function() { callback(req.result || null); };
            req.onerror = function() { callback(null); };
        });
    }

    function clearBgFromDb() {
        openBgDb(function(db) {
            if (!db) return;
            var tx = db.transaction(DB_STORE, "readwrite");
            tx.objectStore(DB_STORE).delete(DB_KEY);
        });
    }

    function applyBgImage(dataUrl) {
        document.body.style.backgroundImage = "url(" + dataUrl + ")";
        document.body.classList.add("has-bg");
        bgPreview.style.backgroundImage = "url(" + dataUrl + ")";
        bgPreview.style.display = "block";
        bgNoImage.style.display = "none";
        bgClearBtn.classList.remove("hidden");
    }

    function clearBgImage() {
        document.body.style.backgroundImage = "";
        document.body.classList.remove("has-bg");
        bgPreview.style.backgroundImage = "";
        bgPreview.style.display = "none";
        bgNoImage.style.display = "";
        bgClearBtn.classList.add("hidden");
        clearBgFromDb();
    }

    function applyOpacity(val) {
        document.documentElement.style.setProperty("--panel-opacity", (val / 100).toString());
        bgOpacityValue.textContent = val + "%";
        bgOpacitySlider.value = val;
        localStorage.setItem("luau-grader-opacity", val);
    }

    bgUploadBtn.addEventListener("click", function() { bgFileInput.click(); });

    var themeUploadBtn = document.getElementById("theme-upload-btn");
    var themeResetBtn = document.getElementById("theme-reset-btn");
    var themeFileInput = document.getElementById("theme-file-input");
    var themeGallery = document.getElementById("theme-gallery");
    var themeOpenFolderBtn = document.getElementById("theme-open-folder-btn");

    var defaultTheme = {
        "bg-base": "#1e1e2e", "bg-surface": "#181825", "bg-deeper": "#11111b", "bg-overlay": "#313244", "border": "#313244",
        "text": "#cdd6f4", "text-dim": "#6c7086", "text-muted": "#45475a", "accent": "#89b4fa", "accent-hover": "#74c7ec",
        "error": "#f38ba8", "warning": "#f9e2af", "info": "#89b4fa", "success": "#a6e3a1",
        "hl-keyword": "#cba6f7", "hl-string": "#a6e3a1", "hl-number": "#fab387", "hl-comment": "#6c7086",
        "hl-literal": "#fab387", "hl-builtin": "#89b4fa", "hl-method": "#f9e2af"
    };

    var logoImg = document.querySelector("#logo img");
    var activeThemeName = null;

    function applyThemeColors(colors, logoPath) {
        for (var key in colors) {
            document.documentElement.style.setProperty("--" + key, colors[key]);
        }
        if (logoImg && logoPath) {
            logoImg.src = logoPath;
        }
    }

    function resetTheme() {
        applyThemeColors(defaultTheme);
        if (logoImg) logoImg.src = "logo.png";
        document.documentElement.style.removeProperty("--logo-filter");
        localStorage.removeItem("luau-grader-theme");
        activeThemeName = null;
        themeResetBtn.classList.add("hidden");
        updateGalleryActiveState();
    }

    function activateTheme(themeJson) {
        applyThemeColors(themeJson.colors, themeJson.logo || null);
        localStorage.setItem("luau-grader-theme", JSON.stringify(themeJson));
        activeThemeName = themeJson.name || null;
        themeResetBtn.classList.remove("hidden");
        updateGalleryActiveState();
    }

    function updateGalleryActiveState() {
        var cards = themeGallery ? themeGallery.querySelectorAll(".theme-card") : [];
        for (var i = 0; i < cards.length; i++) {
            var cardName = cards[i].getAttribute("data-theme-name");
            if (cardName === activeThemeName) {
                cards[i].classList.add("active");
            } else {
                cards[i].classList.remove("active");
            }
        }
    }

    function renderThemeCard(theme) {
        var card = document.createElement("div");
        card.className = "theme-card";
        card.setAttribute("data-theme-name", theme.name);

        var swatches = document.createElement("div");
        swatches.className = "theme-card-swatches";
        var swatchColors = [theme.colors["bg-base"], theme.colors["accent"], theme.colors["text"]];
        for (var i = 0; i < swatchColors.length; i++) {
            var sw = document.createElement("div");
            sw.className = "theme-swatch";
            sw.style.backgroundColor = swatchColors[i];
            swatches.appendChild(sw);
        }
        card.appendChild(swatches);

        var name = document.createElement("span");
        name.className = "theme-card-name";
        name.textContent = theme.name;
        card.appendChild(name);

        var dot = document.createElement("div");
        dot.className = "theme-card-active-dot";
        card.appendChild(dot);

        card.addEventListener("click", function() {
            if (activeThemeName === theme.name) {
                resetTheme();
                showToast("Restored original theme");
            } else {
                activateTheme(theme);
                showToast("Theme '" + theme.name + "' applied!");
            }
        });

        return card;
    }

    function loadThemeGallery() {
        if (!themeGallery) return;
        if (typeof window.__TAURI__ !== "undefined") {
            window.__TAURI__.core.invoke("list_themes").then(function(themes) {
                themeGallery.innerHTML = "";
                for (var i = 0; i < themes.length; i++) {
                    themeGallery.appendChild(renderThemeCard(themes[i]));
                }
                updateGalleryActiveState();
            }).catch(function() {});
        }
    }

    function loadSavedTheme() {
        var saved = localStorage.getItem("luau-grader-theme");
        if (saved) {
            try {
                var json = JSON.parse(saved);
                if (json.colors) {
                    applyThemeColors(json.colors, json.logo || null);
                    activeThemeName = json.name || null;
                }
                themeResetBtn.classList.remove("hidden");
            } catch (e) {}
        }
    }

    if (themeOpenFolderBtn) {
        themeOpenFolderBtn.addEventListener("click", function() {
            if (typeof window.__TAURI__ !== "undefined") {
                window.__TAURI__.core.invoke("open_themes_folder").catch(function() {});
            }
        });
    }

    if (themeUploadBtn) {
        themeUploadBtn.addEventListener("click", function() { themeFileInput.click(); });
    }

    if (themeFileInput) {
        themeFileInput.addEventListener("change", function(e) {
            var file = e.target.files[0];
            if (!file) return;
            var reader = new FileReader();
            reader.onload = function(evt) {
                try {
                    var json = JSON.parse(evt.target.result);
                    if (json.colors) {
                        activateTheme(json);
                        showToast("Theme '" + (json.name || "Custom") + "' applied!");
                    } else {
                        showToast("Invalid theme file: missing 'colors' object");
                    }
                } catch (err) {
                    showToast("Failed to parse JSON file");
                }
                themeFileInput.value = "";
            };
            reader.readAsText(file);
        });
    }

    if (themeResetBtn) {
        themeResetBtn.addEventListener("click", function() {
            resetTheme();
            showToast("Restored original theme");
        });
    }

    loadSavedTheme();
    loadThemeGallery();

    bgFileInput.addEventListener("change", function() {
        var file = bgFileInput.files[0];
        if (!file) return;
        if (file.size > MAX_FILE_SIZE) {
            showToast("Image too large (max 25MB)");
            bgFileInput.value = "";
            return;
        }
        var reader = new FileReader();
        reader.onload = function(e) {
            var dataUrl = e.target.result;
            applyBgImage(dataUrl);
            saveBgToDb(dataUrl);
            showToast("Background image set");
        };
        reader.readAsDataURL(file);
        bgFileInput.value = "";
    });

    bgClearBtn.addEventListener("click", function() {
        clearBgImage();
        showToast("Background cleared");
    });

    bgOpacitySlider.addEventListener("input", function() {
        applyOpacity(parseInt(bgOpacitySlider.value));
    });

    var savedOpacity = localStorage.getItem("luau-grader-opacity");
    if (savedOpacity) {
        applyOpacity(parseInt(savedOpacity));
    } else {
        applyOpacity(90);
    }

    loadBgFromDb(function(dataUrl) {
        if (dataUrl) applyBgImage(dataUrl);
    });

    loadSettings();
    onEditorChange();
})();