Node: compiler/view-loans

Decision: Retire and delete this node, moving the representation choice to compiler/checker-facts, because REF-1 through REF-3 replace continuing region/strength loans and returned views with local reference paths and flow-sensitive validity. The successor retains possible targets, captured offsets and validity through local copies and calls, instead of preserving obsolete loan strength, parent-loan or result-view metadata.
