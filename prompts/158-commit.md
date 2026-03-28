Add owner removal to AccountSettingsPage

- Add removeOwner API client (POST /commands/remove_owner)
- Add delete button for each owner in the owner list
- Disable delete button when only one owner remains
- Show confirmation modal before removing an owner
