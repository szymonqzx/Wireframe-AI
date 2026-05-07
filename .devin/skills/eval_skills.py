#!/usr/bin/env python3
"""
Skill evaluation script for autoimprove.
Measures skill quality, coverage, and consistency using sophisticated pattern recognition.
"""

import os
import re
import sys
import json
import argparse
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from collections import Counter


class SkillEvaluator:
    """Evaluates Wireframe-AI skills against quality, coverage, and consistency standards."""

    # Required sections with equivalents for flexible matching
    REQUIRED_SECTIONS = {
        "purpose": ["purpose", "overview", "what", "about", "description", "mission"],
        "when_to_use": ["when to use", "when to invoke", "when to apply", "trigger", "triggers", "usage", "when not to use"],
        "protocol": ["protocol", "the iron law", "process", "procedure", "method", "workflow", "steps", "the four phases", "the pattern"],
        "steps": ["steps", "step", "workflow", "the iron law", "key patterns", "phase", "protocol"],
        "workflow": ["workflow", "steps", "step", "protocol", "process", "procedure"]
    }

    def __init__(self, skills_dir: Path):
        self.skills_dir = skills_dir
        self.skill_files = self._find_skill_files()

    def _find_skill_files(self) -> List[Path]:
        """Find all SKILL.md files in the skills directory."""
        skill_files = []
        for skill_path in self.skills_dir.rglob("SKILL.md"):
            # Skip autoimprove skill to avoid self-evaluation
            if "autoimprove" not in str(skill_path):
                skill_files.append(skill_path)
        return sorted(skill_files)

    def _read_skill(self, skill_path: Path) -> str:
        """Read skill file content."""
        try:
            with open(skill_path, 'r', encoding='utf-8') as f:
                return f.read()
        except Exception as e:
            print(f"Error reading {skill_path}: {e}", file=sys.stderr)
            return ""

    def _check_frontmatter(self, content: str) -> Tuple[float, List[str]]:
        """Check for proper YAML frontmatter."""
        issues = []
        score = 1.0

        # Check for frontmatter markers
        if not content.startswith('---'):
            issues.append("Missing YAML frontmatter opening")
            score -= 0.3
        else:
            # Check for closing marker
            first_section = content.split('---', 2)
            if len(first_section) < 3:
                issues.append("Missing YAML frontmatter closing")
                score -= 0.3
            else:
                # Check for required frontmatter fields
                frontmatter = first_section[1]
                required_fields = ['name', 'description', 'triggers']
                for field in required_fields:
                    if field not in frontmatter.lower():
                        issues.append(f"Missing frontmatter field: {field}")
                        score -= 0.1

        return max(score, 0.0), issues

    def _check_coverage(self, content: str) -> Tuple[float, List[str]]:
        """Check if skill has required sections and coverage elements."""
        content_lower = content.lower()
        missing = []
        present = 0

        # Check for required sections using equivalents
        for section, equivalents in self.REQUIRED_SECTIONS.items():
            if any(eq in content_lower for eq in equivalents):
                present += 1
            else:
                missing.append(section)

        # Check for skill structure elements
        has_purpose = any(term in content_lower for term in ["purpose", "what", "overview", "about", "description"])
        has_trigger = any(term in content_lower for term in ["trigger", "when to use", "when to invoke", "when to apply"])
        has_protocol = any(term in content_lower for term in ["protocol", "steps", "workflow", "how to", "process", "phase"])
        has_integration = any(term in content_lower for term in ["integration", "related", "see also", "integrates with"])

        coverage_score = present / len(self.REQUIRED_SECTIONS)

        # Bonus points for structure
        if has_purpose:
            coverage_score += 0.05
        if has_trigger:
            coverage_score += 0.05
        if has_protocol:
            coverage_score += 0.05
        if has_integration:
            coverage_score += 0.05

        return min(coverage_score, 1.0), missing

    def _check_consistency(self, content: str, skill_path: Path) -> Tuple[float, List[str]]:
        """Check formatting and structural consistency."""
        issues = []
        score = 1.0

        lines = content.split('\n')

        # Check for proper markdown headers
        header_pattern = re.compile(r'^#+\s')
        headers = [line for line in lines if header_pattern.match(line)]

        if not headers:
            issues.append("No markdown headers found")
            score -= 0.2

        # Check for code block formatting
        if '```' not in content:
            issues.append("No code blocks found")
            score -= 0.15

        # Check for list formatting
        if not any(line.strip().startswith(('- ', '* ', '+ ')) for line in lines):
            issues.append("No lists found")
            score -= 0.1

        # Check for table formatting (good skills often have comparison tables)
        if '|' not in content:
            issues.append("No tables found (comparison tables improve clarity)")
            score -= 0.05

        # Check for proper line endings (no trailing whitespace)
        trailing_whitespace = [i+1 for i, line in enumerate(lines) if line != line.rstrip()]
        if trailing_whitespace:
            issues.append(f"Trailing whitespace on lines: {trailing_whitespace[:5]}...")
            score -= 0.05

        # Check for consistent header hierarchy
        header_levels = [len(re.match(r'^#+', line).group()) for line in headers if re.match(r'^#+', line)]
        if header_levels and max(header_levels) > 4:
            issues.append("Header depth exceeds 4 levels")
            score -= 0.05

        return max(score, 0.0), issues

    def _check_actionability(self, content: str) -> Tuple[float, List[str]]:
        """Check if skill provides actionable, specific guidance."""
        issues = []
        score = 1.0

        content_lower = content.lower()

        # Check for action verbs
        action_verbs = ["use", "run", "execute", "implement", "fix", "add", "create", "check", "verify", "test", "follow", "apply", "invoke"]
        action_count = sum(1 for verb in action_verbs if verb in content_lower)
        if action_count < 3:
            issues.append("Could be more actionable (few action verbs)")
            score -= 0.1
        elif action_count < 5:
            issues.append("Could be more actionable")
            score -= 0.05

        # Check for step-by-step guidance
        step_patterns = [r'step \d+', r'phase \d+', r'\d+\.', r'first', r'then', r'next', r'finally']
        has_steps = any(re.search(pattern, content_lower) for pattern in step_patterns)
        if not has_steps:
            issues.append("Missing step-by-step guidance")
            score -= 0.15

        # Check for specific examples
        example_patterns = ["example", "for example", "e.g.", "sample", "pattern", "use case"]
        has_examples = any(pattern in content_lower for pattern in example_patterns)
        if not has_examples:
            issues.append("Could be more specific with examples")
            score -= 0.1

        # Check for anti-patterns or common mistakes (high-quality skills have these)
        anti_patterns = ["anti-pattern", "common mistake", "don't", "avoid", "wrong", "bad", "never", "forbidden"]
        has_anti_patterns = any(pattern in content_lower for pattern in anti_patterns)
        if not has_anti_patterns:
            issues.append("Missing anti-patterns or common mistakes")
            score -= 0.05

        return max(score, 0.0), issues

    def _check_clarity(self, content: str) -> Tuple[float, List[str]]:
        """Check for clarity, conciseness, and effectiveness."""
        issues = []
        score = 1.0

        content_lower = content.lower()
        word_count = len(content.split())

        # Check for excessive verbosity
        if word_count > 2000:
            issues.append(f"Skill is very long ({word_count} words)")
            score -= 0.1
        elif word_count > 1000:
            issues.append(f"Skill is long ({word_count} words)")
            score -= 0.05

        # Check for repetition
        sentences = content.split('.')
        unique_sentences = set(sentences)
        if len(sentences) > 10 and len(unique_sentences) / len(sentences) < 0.8:
            issues.append("Potential repetition detected")
            score -= 0.15

        # Check for clarity indicators
        clarity_indicators = ["specific", "clear", "explicit", "step", "example", "pattern"]
        if not any(indicator in content_lower for indicator in clarity_indicators):
            issues.append("Could be more specific with examples")
            score -= 0.05

        # Check for vague language
        vague_phrases = ["should", "might", "could", "possibly", "perhaps", "maybe"]
        vague_count = sum(1 for phrase in vague_phrases if phrase in content_lower)
        if vague_count > 5:
            issues.append(f"Uses vague language ({vague_count} instances)")
            score -= 0.1

        return max(score, 0.0), issues

    def _check_structure_quality(self, content: str) -> Tuple[float, List[str]]:
        """Check for high-quality structural elements."""
        issues = []
        score = 1.0

        content_lower = content.lower()

        # Check for comparison tables (wrong vs right, good vs bad)
        has_comparison = '|' in content and any(term in content_lower for term in ["wrong", "right", "bad", "good", "don't", "do"])
        if not has_comparison:
            issues.append("Missing comparison tables (wrong vs right)")
            score -= 0.05

        # Check for integration points
        has_integration = any(term in content_lower for term in ["integration", "integrates with", "related", "see also"])
        if not has_integration:
            issues.append("Missing integration points to other skills")
            score -= 0.05

        # Check for clear hierarchy (nested sections)
        headers = re.findall(r'^(#+)\s', content, re.MULTILINE)
        if len(headers) < 3:
            issues.append("Limited section hierarchy")
            score -= 0.05

        # Check for code examples in code blocks
        code_blocks = re.findall(r'```(\w+)?', content)
        if not code_blocks:
            issues.append("No code examples found")
            score -= 0.1

        return max(score, 0.0), issues

    def _calculate_length_penalty(self, content: str, avg_tokens: float, max_tokens: float) -> Tuple[float, List[str]]:
        """Calculate length penalty based on token count relative to other skills.
        No hardcoded thresholds - purely statistical calculation.
        """
        issues = []
        word_count = len(content.split())

        # Calculate penalty based on deviation from average
        # Skills at average get no penalty
        # Skills significantly longer than average get penalty
        # Skills significantly shorter than average get slight penalty (too brief)

        if avg_tokens == 0:
            return 0.0, []

        # Calculate ratio of this skill's length to average
        length_ratio = word_count / avg_tokens

        # Penalty based on how much longer than average
        # Linear penalty: 0 penalty at average, increasing penalty for longer skills
        if length_ratio <= 0.5:
            # Too brief (less than 50% of average)
            penalty = 0.15
            issues.append(f"Skill is too brief ({word_count} words vs avg {avg_tokens:.0f})")
        elif length_ratio <= 0.8:
            # Slightly brief (50-80% of average)
            penalty = 0.05
            issues.append(f"Skill is slightly brief ({word_count} words vs avg {avg_tokens:.0f})")
        elif length_ratio <= 1.2:
            # Near average (80-120% of average) - optimal range
            penalty = 0.0
        elif length_ratio <= 1.5:
            # Slightly long (120-150% of average)
            penalty = 0.05
            issues.append(f"Skill is slightly long ({word_count} words vs avg {avg_tokens:.0f})")
        elif length_ratio <= 2.0:
            # Moderately long (150-200% of average)
            penalty = 0.15
            issues.append(f"Skill is moderately long ({word_count} words vs avg {avg_tokens:.0f})")
        else:
            # Very long (more than 2x average)
            penalty = 0.3
            issues.append(f"Skill is very long ({word_count} words vs avg {avg_tokens:.0f})")

        return penalty, issues

    def _calculate_quality_score(self, content: str, skill_path: Path) -> Tuple[float, List[str]]:
        """Calculate overall quality score from multiple dimensions."""
        all_issues = []

        # Get scores from each dimension
        frontmatter_score, frontmatter_issues = self._check_frontmatter(content)
        actionability_score, actionability_issues = self._check_actionability(content)
        clarity_score, clarity_issues = self._check_clarity(content)
        structure_score, structure_issues = self._check_structure_quality(content)

        all_issues.extend(frontmatter_issues)
        all_issues.extend(actionability_issues)
        all_issues.extend(clarity_issues)
        all_issues.extend(structure_issues)

        # Weighted average (frontmatter is critical, others are important)
        # Note: Length penalty is now handled in overall score calculation, not here
        quality_score = (
            frontmatter_score * 0.3 +
            actionability_score * 0.3 +
            clarity_score * 0.2 +
            structure_score * 0.2
        )

        return quality_score, all_issues

    def evaluate_skill(self, skill_path: Path, avg_tokens: float, max_tokens: float, test_only: bool = False) -> Dict:
        """Evaluate a single skill file."""
        content = self._read_skill(skill_path)
        if not content:
            return {
                "path": str(skill_path),
                "error": "Could not read file",
                "coverage_score": 0.0,
                "consistency_score": 0.0,
                "quality_score": 0.0,
                "overall_score": 0.0,
                "critical_errors": 1
            }

        frontmatter_score, _ = self._check_frontmatter(content)
        coverage_score, coverage_missing = self._check_coverage(content)
        consistency_score, consistency_issues = self._check_consistency(content, skill_path)
        quality_score, quality_issues = self._calculate_quality_score(content, skill_path)

        # Calculate token count for this skill
        token_count = len(content.split())

        # Get length issues for reporting (but not for scoring)
        _, length_issues = self._calculate_length_penalty(content, avg_tokens, max_tokens)

        # Calculate overall score divided by token count
        # SKILL_SCORE = (coverage + consistency + quality) / token_count * 10000
        # This gives: perfect scores (3.0) / 1000 tokens * 10000 = 30%
        # Longer skills = lower score, shorter skills = higher score
        #
        # ANTI-GAMING: Prevent mindless content removal by enforcing minimum coverage
        # If coverage < 0.8 (80%), apply heavy penalty to prevent gaming the system
        if token_count > 0:
            # Sum of scores (coverage + consistency + quality)
            score_sum = coverage_score + consistency_score + quality_score
            # Divide by token count and multiply by constant to get percentage
            overall_score = (score_sum / token_count) * 10000

            # Anti-gaming: Enforce minimum coverage threshold
            # Skills with coverage < 80% are considered incomplete and penalized heavily
            if coverage_score < 0.8:
                # Apply severe penalty for insufficient coverage
                # This prevents mindless content removal to boost token-based score
                overall_score *= 0.3  # 70% penalty for low coverage
                all_issues.append(f"Coverage below threshold ({coverage_score:.2f} < 0.8) - content removal detected")
            elif coverage_score < 0.9:
                # Moderate penalty for borderline coverage
                overall_score *= 0.7  # 30% penalty for borderline coverage
                all_issues.append(f"Coverage borderline ({coverage_score:.2f} < 0.9)")
        else:
            # Fallback to weighted average if no token data
            overall_score = (coverage_score * 0.4 + consistency_score * 0.3 + quality_score * 0.3) * 100

        all_issues = coverage_missing + consistency_issues + quality_issues + length_issues
        critical_errors = 1 if not content or coverage_score < 0.3 or frontmatter_score < 0.5 else 0

        result = {
            "path": str(skill_path),
            "coverage_score": coverage_score,
            "consistency_score": consistency_score,
            "quality_score": quality_score,
            "overall_score": overall_score,
            "token_count": token_count,
            "issues": all_issues,
            "critical_errors": critical_errors,
            "coverage_missing": coverage_missing
        }

        return result

    def evaluate_all(self, test_only: bool = False) -> Dict:
        """Evaluate all skills and return aggregate results."""
        # First pass: calculate token counts for all skills
        token_counts = []
        for skill_path in self.skill_files:
            content = self._read_skill(skill_path)
            if content:
                token_count = len(content.split())
                token_counts.append(token_count)

        # Calculate statistics
        if not token_counts:
            return {
                "error": "No skill files found",
                "SKILL_SCORE": 0.0,
                "CRITICAL_ERRORS": 1
            }

        avg_tokens = sum(token_counts) / len(token_counts)
        max_tokens = max(token_counts)

        # Second pass: evaluate all skills with token statistics
        results = []
        total_coverage = 0.0
        total_consistency = 0.0
        total_quality = 0.0
        total_overall = 0.0
        critical_errors = 0

        for skill_path in self.skill_files:
            result = self.evaluate_skill(skill_path, avg_tokens, max_tokens, test_only)
            results.append(result)

            total_coverage += result["coverage_score"]
            total_consistency += result["consistency_score"]
            total_quality += result["quality_score"]
            total_overall += result["overall_score"]
            critical_errors += result["critical_errors"]

        if not results:
            return {
                "error": "No skill files found",
                "SKILL_SCORE": 0.0,
                "CRITICAL_ERRORS": 1
            }

        num_skills = len(results)
        avg_coverage = total_coverage / num_skills
        avg_consistency = total_consistency / num_skills
        avg_quality = total_quality / num_skills
        avg_overall = total_overall / num_skills

        return {
            "num_skills": num_skills,
            "avg_coverage": avg_coverage,
            "avg_consistency": avg_consistency,
            "avg_quality": avg_quality,
            "SKILL_SCORE": avg_overall,
            "CRITICAL_ERRORS": critical_errors,
            "avg_tokens": avg_tokens,
            "max_tokens": max_tokens,
            "detailed_results": results if not test_only else []
        }


def main():
    parser = argparse.ArgumentParser(description="Evaluate Wireframe-AI skills")
    parser.add_argument("--test-only", action="store_true", help="Only run tests without scoring")
    args = parser.parse_args()

    # Find skills directory
    script_dir = Path(__file__).parent
    skills_dir = script_dir

    evaluator = SkillEvaluator(skills_dir)
    results = evaluator.evaluate_all(test_only=args.test_only)

    if "error" in results:
        print(f"ERROR: {results['error']}", file=sys.stderr)
        sys.exit(1)

    if args.test_only:
        # Test mode: just check that evaluation works
        if results["CRITICAL_ERRORS"] == 0:
            print("[OK] All skills pass basic checks")
            sys.exit(0)
        else:
            print(f"[FAIL] {results['CRITICAL_ERRORS']} critical errors found")
            sys.exit(1)
    else:
        # Scoring mode: output the score (already calculated as percentage)
        print(f"SKILL_SCORE: {results['SKILL_SCORE']:.2f}%")
        print(f"CRITICAL_ERRORS: {results['CRITICAL_ERRORS']}")
        print(f"Skills evaluated: {results['num_skills']}")
        print(f"Average tokens: {results['avg_tokens']:.0f}")
        print(f"Max tokens: {results['max_tokens']:.0f}")
        print(f"Average coverage: {results['avg_coverage'] * 100:.2f}%")
        print(f"Average consistency: {results['avg_consistency'] * 100:.2f}%")
        print(f"Average quality: {results['avg_quality'] * 100:.2f}%")

        # Show worst performing skills
        if results.get("detailed_results"):
            sorted_results = sorted(results["detailed_results"], key=lambda x: x["overall_score"])
            print("\nBottom 5 skills:")
            for result in sorted_results[:5]:
                print(f"  {result['path']}: {result['overall_score']:.2f}% ({result['token_count']} tokens)")
                if result["issues"]:
                    print(f"    Issues: {', '.join(result['issues'][:3])}")

            # Show critical errors
            critical_skills = [r for r in results["detailed_results"] if r["critical_errors"] == 1]
            if critical_skills:
                print("\nCritical errors (coverage < 0.3 or missing frontmatter):")
                for result in critical_skills:
                    coverage_percent = result['coverage_score'] * 100
                    print(f"  {result['path']}: {coverage_percent:.2f}%")
                    print(f"    Missing: {', '.join(result['coverage_missing'])}")

        sys.exit(0)


if __name__ == "__main__":
    main()
