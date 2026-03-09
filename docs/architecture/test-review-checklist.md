# Test Review Checklist

Use this checklist during review:

1. Does this change add or update at least one failure-path test?
2. Does it assert output behavior, not just success status?
3. Does it assert exit-code behavior for failures?
4. If stateful, does it include filesystem failure/corruption scenarios?
5. If parser-related, does it include malformed input coverage?
6. If plugin lifecycle-related, does it include rollback coverage?
7. If config mutation-related, does it include corruption-resistance coverage?
8. Are snapshots used only where regression value is clear?
9. Are test names specific about behavior and failure mode?
10. Would this test fail if a realistic regression occurs?
