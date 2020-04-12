## MIDI Gate

This plugin demonstrates:
* Receiving MIDI input
* Processing audio based on MIDI events with sample accuracy
* Supporting MIDI programs which the host can control/automate, or present a user interface for with human readable labels

A key concept of LV2 that is introduced with this plugin is URID. As you've learned before, many things in the LV2 ecosystem are identified by URIs. However, comparing URIs isn't nescessarily fast and the time it takes to compare URIs rises with their length. Instead, every known URI is mapped to number, a so-called URID, which is used instead of the full URI when time and space is valuable. This mapping is done by the host, which also assures that the mappings are consistent across plugins. Therefore, URIDs are also used for host-plugin or plugin-plugin communication.