# XR819 Reverse engineering

Don't search the web for details on xr819. We're reverse engineering this black box.

# Development guidelines

It's ok to consider jj commits as checkpoints, but if some hardware isn't working, the previous checkpoint is not a good endpoint. 
This means you need to continue looking by looking whay your rust code does wrong or what vendor decompiled code does right.
Make jj commits when you're 100% sure you made an improvement.
Document the process of what worked and didn't along the way.
You seem to be confused by all of the test cargo features. Drop them once you're done investigating to keep it at a mimimum.

