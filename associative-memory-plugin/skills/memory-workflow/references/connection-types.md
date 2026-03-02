# Comprehensive Connection Types Reference

Extensive list of semantic relationship types for associative memory connections.

## Causality

| Type | Meaning | Example |
|------|---------|---------|
| causes | A produces B as effect | "memory leak causes performance degradation" |
| prevents | A stops B from occurring | "validation prevents invalid data" |
| triggers | A initiates B | "button click triggers API call" |
| enables | A makes B possible | "authentication enables secure access" |
| blocks | A stops B | "firewall blocks malicious traffic" |
| results_in | A leads to outcome B | "optimization results_in faster load" |

## Structure

| Type | Meaning | Example |
|------|---------|---------|
| is_a | A is instance/type of B | "JWT is_a token type" |
| part_of | A is component of B | "wheel part_of car" |
| contains | A includes B | "module contains functions" |
| includes | A has B as member | "framework includes utilities" |
| composed_of | A made up of B | "system composed_of microservices" |

## Dependency

| Type | Meaning | Example |
|------|---------|---------|
| uses | A utilizes B | "API uses Redis cache" |
| requires | A needs B | "build requires Node.js" |
| depends_on | A relies on B | "frontend depends_on backend" |
| needs | A must have B | "deployment needs credentials" |
| relies_on | A can't function without B | "service relies_on database" |

## Provenance

| Type | Meaning | Example |
|------|---------|---------|
| created_by | A authored by B | "design created_by architect" |
| authored_by | A written by B | "paper authored_by Smith" |
| stated_by | A claimed by B | "fact stated_by expert" |
| from | A originates from B | "data from sensor" |
| source | A sourced from B | "quote source paper" |

## Logic

| Type | Meaning | Example |
|------|---------|---------|
| contradicts | A conflicts with B | "Paper A contradicts Paper B" |
| supports | A reinforces B | "evidence supports hypothesis" |
| extends | A builds on B | "v2 extends v1" |
| refines | A improves B | "patch refines algorithm" |
| implies | A suggests B | "symptom implies diagnosis" |

## Temporal

| Type | Meaning | Example |
|------|---------|---------|
| precedes | A comes before B | "design precedes implementation" |
| follows | A comes after B | "testing follows coding" |
| supersedes | A replaces B | "v2 supersedes v1" |
| evolved_into | A became B | "prototype evolved_into product" |

## Comparison

| Type | Meaning | Example |
|------|---------|---------|
| better_than | A superior to B | "SSD better_than HDD" |
| worse_than | A inferior to B | "latency worse_than expected" |
| similar_to | A resembles B | "React similar_to Vue" |
| different_from | A unlike B | "NoSQL different_from SQL" |
| equivalent_to | A same as B | "1GB equivalent_to 1024MB" |

## Implementation

| Type | Meaning | Example |
|------|---------|---------|
| implemented_as | A realized through B | "cache implemented_as Redis" |
| implemented_by | A created using B | "UI implemented_by React" |
| uses_technology | A built with B | "app uses_technology Node.js" |
| powered_by | A runs on B | "search powered_by Elasticsearch" |

## Association

| Type | Meaning | Example |
|------|---------|---------|
| related_to | A connected to B | "topic related_to field" |
| associated_with | A linked with B | "error associated_with timeout" |
| connected_to | A tied to B | "issue connected_to PR" |

## Use Sparingly

These are vague - prefer specific types:
- `linked_to` → Use `depends_on`, `uses`, etc.
- `related_to` → Use `part_of`, `similar_to`, etc.
- `connected_to` → Use `causes`, `requires`, etc.

## Custom Types

Create domain-specific types as needed:
- `diagnosed_as` (medical)
- `cited_by` (academic)
- `deployed_to` (DevOps)
- `assigned_to` (project management)

## Tips

1. **Be specific**: "depends_on" > "related_to"
2. **Be directional**: Consider if reverse makes sense
3. **Be consistent**: Same relationship = same type
4. **Be semantic**: Type should convey meaning
5. **Be minimal**: Don't create unnecessary types
