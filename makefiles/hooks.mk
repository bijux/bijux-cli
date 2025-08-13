# Git Hooks
GIT_HOOKS_DIR 	:= .git/hooks
LOCAL_HOOKS_DIR := scripts/git-hooks

.PHONY: install-git-hooks

install-git-hooks:
	@echo "→ Installing Git hooks..."
	@mkdir -p $(GIT_HOOKS_DIR)
	@for hook in pre-push pre-commit prepare-commit-msg; do \
	  if [ -f "$(LOCAL_HOOKS_DIR)/$$hook" ]; then \
	    if [ -L "$(GIT_HOOKS_DIR)/$$hook" ]; then \
	      echo "  ✔ $$hook already linked — skipping"; \
	    else \
	      echo "  Linking $$hook"; \
	      ln -sf "../../$(LOCAL_HOOKS_DIR)/$$hook" "$(GIT_HOOKS_DIR)/$$hook"; \
	      chmod +x "$(LOCAL_HOOKS_DIR)/$$hook"; \
	    fi; \
	  fi; \
	done
	@echo "✔ Git hooks installed"
