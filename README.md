A basic cli tool to manage (configuration) files based on profiles.

It works by keeping a list of managed files. Adding a file to a profile means creating two copies of it. One original, and one specific to the profile.
When activating a profile, all files managed by it will be replaced by the profile-specific one.
When deactivating all managed files of all profiles will be replaced by their previously created original versions.

Before any activation, adding or removal all profiles will be activated to assure a defined state.

All file-paths will automatically be made absolute. 
