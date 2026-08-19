Write-Host "=========================================="
Write-Host "🌙 VYOUM NIGHT SCHOOL BATCH SCRIPT 🌙"
Write-Host "=========================================="
Write-Host "Injecting Advanced Physics (Level 3)..."
python -m subconscious.wikidata_tutor --syllabus subconscious/curriculum/physics_level_3.json

Write-Host "Injecting Advanced Math (Level 2)..."
python -m subconscious.wikidata_tutor --syllabus subconscious/curriculum/math_level_2.json

Write-Host "Injecting Biology (Level 1: Fundamentals)..."
python -m subconscious.wikidata_tutor --syllabus subconscious/curriculum/biology_level_1.json

Write-Host "Injecting Biology (Level 2: Intermediate)..."
python -m subconscious.wikidata_tutor --syllabus subconscious/curriculum/biology_level_2.json

Write-Host "Injecting Biology (Level 3: Advanced)..."
python -m subconscious.wikidata_tutor --syllabus subconscious/curriculum/biology_level_3.json

Write-Host "=========================================="
Write-Host "✅ NIGHT SCHOOL INJECTION COMPLETE!"
Write-Host "VYOUM Engine is now digesting the universe."
Write-Host "=========================================="
