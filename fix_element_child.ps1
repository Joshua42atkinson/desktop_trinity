# Fix ElementChild imports across all affected files
$files = @(
    # Components
    "frontend/src/components/design_process.rs",
    "frontend/src/components/ethical_legal.rs",
    "frontend/src/components/formative_eval.rs",
    "frontend/src/components/learner_analysis.rs",
    "frontend/src/components/motivational_design.rs",
    "frontend/src/components/systems_thinking.rs",
    "frontend/src/components/tech_analysis.rs",
    "frontend/src/components/tech_skills.rs",
    "frontend/src/components/theoretical_synthesis.rs",
    "frontend/src/components/visual_design.rs",
    # Pages
    "frontend/src/pages/foundations.rs",
    "frontend/src/pages/planning.rs",
    "frontend/src/pages/design.rs",
    "frontend/src/pages/evaluation.rs",
    "frontend/src/pages/home.rs",
    "frontend/src/pages/systems_thinking.rs"
)

$basePath = "c:/Users/Trinity/Documents/daydream/Day_Dream"
$count = 0

foreach ($file in $files) {
    $fullPath = Join-Path $basePath $file
    if (Test-Path $fullPath) {
        $content = Get-Content $fullPath -Raw
        
        # Check if ElementChild import already exists
        if ($content -notmatch "ElementChild") {
            # Find the first use statement and add ElementChild after it
            if ($content -match "use leptos::prelude::\*;") {
                $content = $content -replace "(use leptos::prelude::\*;)", "`$1`r`nuse leptos::prelude::ElementChild;"
                Set-Content $fullPath $content -NoNewline
                Write-Host "Fixed: $file" -ForegroundColor Green
                $count++
            }
            elseif ($content -match "use leptos::\*;") {
                # Update old import style first
                $content = $content -replace "use leptos::\*;", "use leptos::prelude::*;`r`nuse leptos::prelude::ElementChild;"
                Set-Content $fullPath $content -NoNewline
                Write-Host "Updated and fixed: $file" -ForegroundColor Cyan
                $count++
            }
        }
        else {
            Write-Host "Skipped (already has ElementChild): $file" -ForegroundColor Yellow
        }
    }
    else {
        Write-Host "Not found: $file" -ForegroundColor Red
    }
}

Write-Host "`nTotal files fixed: $count" -ForegroundColor Green
