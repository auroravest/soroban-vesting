# Security Policy

Security reports help protect contract users and their funds. Please report
suspected vulnerabilities privately and allow time for a fix before sharing
technical details publicly.

## Reporting a Vulnerability

1. Open the repository's **Security** tab and select **Report a vulnerability**
   to create a private GitHub security advisory.
2. If private reporting is unavailable, open a minimal public issue asking the
   maintainers for a private contact channel. Do not include exploit details,
   secrets, addresses at risk, or a proof of concept in that issue.
3. Include the affected commit or release, impact, reproduction steps, and any
   suggested mitigation. A minimal proof of concept is welcome when it can be
   run safely against a local or test environment.

Do not open a public pull request containing an unpatched vulnerability before
coordinating disclosure with the maintainers.

## Response Targets

The project aims to:

- acknowledge a complete report within 3 business days;
- complete an initial severity assessment within 7 business days;
- provide a status update at least every 14 days until resolution; and
- coordinate a disclosure date after a fix is available.

Complex reports may require more time. If a target cannot be met, the
maintainers should communicate a revised timeline through the private advisory.

## Scope

The following are in scope:

- the Soroban vesting contract in `contracts/vesting`;
- authorization, custody, vesting, claim, and revocation behavior;
- arithmetic, storage, and event integrity;
- repository build, test, and release automation that can affect published
  contract artifacts; and
- security-sensitive documentation or configuration maintained in this
  repository.

Only test against code and accounts you own or are explicitly authorized to
use. Prefer local tests, simulations, or Stellar testnet deployments created
for the report. Never interact with real user funds.

## Out of Scope

The following are out of scope:

- social engineering, phishing, or physical attacks;
- denial-of-service testing that degrades shared services;
- attacks against GitHub, Stellar, wallets, token issuers, or other third-party
  systems outside this repository;
- automated scans without a demonstrated, repository-specific impact;
- reports that only identify missing best practices without a security impact;
- vulnerabilities in unsupported dependencies when this project is not
  exploitable; and
- disclosure of secrets or personal data obtained without authorization.

## Researcher Expectations

Act in good faith, minimize access to data, stop testing if unintended access
occurs, and delete any data collected during validation. The project will not
pursue action against research that follows this policy, stays within scope,
and avoids privacy violations, service disruption, and financial harm.

Rewards are not guaranteed by this policy. Any campaign eligibility or reward
decision follows the terms of the applicable contribution program.
