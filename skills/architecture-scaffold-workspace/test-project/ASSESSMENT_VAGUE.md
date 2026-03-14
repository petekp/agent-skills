# TaskFlow Quick Review

The code works but it's getting unwieldy. The main engine class does too much. Consider extracting some of the functionality into separate modules — especially the data layer and maybe the notification stuff. The types could probably be organized better too.

Some general suggestions:
- Think about separating concerns
- The persistence logic shouldn't be mixed in with business logic
- Maybe add some abstractions for the external-facing parts
