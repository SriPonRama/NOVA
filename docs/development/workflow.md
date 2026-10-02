# Development Workflow

## 1. Plan
Understand the requirement before modifying code. Read `NOVA-SPEC.md`.

## 2. Implement
Ensure modular separation (UI vs. Native). Keep secrets in `.env` and never commit them.

## 3. Test
- Frontend: `npm run build`
- Native: `cargo check` inside `/src-tauri`
- Development: `npm run tauri dev`

## 4. Review
- Check `git status` and `git diff` for secrets or build artifacts.

## 5. Commit
Use conventional commits: `feat:`, `fix:`, `chore:`, `docs:`, `test:`.

## 6. Push
Push safely to the remote repository. No force pushes.
