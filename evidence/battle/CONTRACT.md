# Battle Evidence Contract

Battle evidence is release-blocking proof for trust properties.

Requirements:
- Every battle scenario must map to at least one battle trust property.
- Every release-blocking battle scenario must protect at least one top trust property.
- Every top trust property must be covered by at least one battle scenario.
- Battle scenarios mapping more than three trust properties are considered overloaded and must be split.
- Every battle scenario must be executable by automated consumers.
- Battle evidence metadata must declare owner, consumer surfaces, and stable scenario id.
