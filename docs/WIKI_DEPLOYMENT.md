# GitHub Wiki Deployment Guide

This document explains how to deploy the documentation to GitHub Wiki.

## Automatic Deployment

The documentation is automatically deployed to GitHub Wiki when:
- Changes are pushed to `docs/` directory on main/master branch
- The workflow is manually triggered from GitHub Actions

## Setup Instructions

### 1. Enable GitHub Wiki

1. Go to your repository Settings
2. Scroll to "Features" section
3. Check "Wikis" to enable

### 2. GitHub Actions Setup

The workflow file `.github/workflows/update-wiki.yml` is already configured. No additional setup needed.

### 3. First Deployment

#### Option A: Manual Trigger
1. Go to Actions tab in your repository
2. Select "Update GitHub Wiki" workflow
3. Click "Run workflow"
4. Select branch and optionally customize commit message
5. Click "Run workflow" button

#### Option B: Push Changes
Simply push changes to any file in the `docs/` directory:
```bash
git add docs/
git commit -m "Update documentation"
git push origin main
```

## How It Works

The GitHub Actions workflow:
1. Clones the wiki repository (separate git repo at `github.com/your-org/twin.wiki.git`)
2. Clears existing wiki content
3. Copies all files from `docs/` directory
4. Creates navigation sidebar (`_Sidebar.md`)
5. Creates footer (`_Footer.md`)
6. Commits and pushes changes to wiki

## File Mapping

| Source (docs/) | Wiki Page | URL |
|---------------|-----------|-----|
| `Home.md` | Home page | `/wiki` |
| `Getting-Started-*.md` | Getting Started section | `/wiki/Getting-Started-*` |
| `Architecture-*.md` | Architecture section | `/wiki/Architecture-*` |
| `Testing-*.md` | Testing section | `/wiki/Testing-*` |
| `Development-Guides-*.md` | Development section | `/wiki/Development-Guides-*` |
| `Deployment-*.md` | Deployment section | `/wiki/Deployment-*` |
| `JA-*.md` | Japanese version | `/wiki/JA-*` |

## Wiki Structure

The wiki automatically gets:
- **Navigation sidebar** (`_Sidebar.md`) - Left navigation menu
- **Footer** (`_Footer.md`) - Bottom of every page
- **Home page** (`Home.md`) - Landing page

## Troubleshooting

### Wiki not updating?
- Check Actions tab for workflow runs
- Ensure Wiki is enabled in repository settings
- Verify `docs/` directory exists and has `.md` files

### Permission errors?
- The workflow needs `contents: write` permission
- This is already configured in the workflow file

### Manual update needed?
1. Go to Actions → Update GitHub Wiki
2. Run workflow with "Force update" option

## Local Testing

To test wiki structure locally:
```bash
# Clone wiki repository
git clone https://github.com/your-org/twin.wiki.git

# Copy docs to wiki
cp -r docs/* twin.wiki/

# View with any markdown viewer
```

## Maintaining Documentation

1. Edit files in `docs/` directory
2. Follow naming convention: `Category-SubCategory-PageName.md`
3. Use `JA-` prefix for Japanese pages
4. Commit and push - wiki updates automatically

## Wiki Features

GitHub Wiki provides:
- Automatic table of contents
- Search functionality
- Revision history
- Clone capability for offline viewing
- Custom sidebar and footer