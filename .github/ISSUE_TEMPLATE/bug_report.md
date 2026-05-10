name: Bug report
description: Create a report to help us improve
labels: ["bug"]
body:
  - type: markdown
    attributes:
      value: |
        Thanks for taking the time to fill out this bug report!
  - type: textarea
    id: description
    attributes:
      label: Describe the bug
      description: A clear and concise description of what the bug is.
    validations:
      required: true
  - type: textarea
    id: reproduction
    attributes:
      label: Reproduction Steps
      description: How do we reproduce this? (e.g. provide the CSS and the command used)
      placeholder: |
        1. Create file `test.css` with content ...
        2. Run `css2tw convert test.css`
        3. See error ...
    validations:
      required: true
  - type: textarea
    id: expected-behavior
    attributes:
      label: Expected behavior
      description: A clear and concise description of what you expected to happen.
    validations:
      required: true
  - type: input
    id: version
    attributes:
      label: css2tw version
      description: What version of css2tw are you using?
    validations:
      required: true
  - type: textarea
    id: additional-context
    attributes:
      label: Additional context
      description: Add any other context about the problem here.
