# Fix all files that need ElementChild import
$files = @(
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
    "frontend/src/pages/foundations.rs",
    "frontend/src/pages/planning.rs",
    "frontend/src/pages/design.rs",
    "frontend/src/pages/evaluation.rs",
    "frontend/src/pages/home.rs",
    "frontend/src/pages/systems_thinking.rs"
)

foreach ($file in $files) {
    $fullPath = "c:/Users/Trinity/Documents/daydream/Day_Dream/$file"
    if (Test-Path $fullPath) {
        $content = Get-Content $fullPath -Raw
        if ($content -match "use leptos::\*;" -and $content -notmatch "ElementChild") {
            $content = $content -replace "use leptos::\*;", "use leptos::prelude::*;`nuse leptos::prelude::ElementChild;"
            Set-Content $fullPath $content -NoNewline
            Write-Host "Fixed: $file"
        } elseif ($content -match "use leptos::prelude::\*;" -and $content -notmatch "ElementChild") {
            $content = $content -replace "(use leptos::prelude::\*;)", "`$1`nuse leptos::prelude::ElementChild;"
            Set-Content $fullPath $content -NoNewline
            Write-Host "Fixed: $file"
        }
    }
}

Write-Host "Done!"
