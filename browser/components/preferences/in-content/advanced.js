/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

/* import-globals-from preferences.js */

// Load DownloadUtils module for convertByteUnits
Components.utils.import("resource://gre/modules/DownloadUtils.jsm");
Components.utils.import("resource://gre/modules/LoadContextInfo.jsm");
Components.utils.import("resource://gre/modules/XPCOMUtils.jsm");

var gAdvancedPane = {
  _inited: false,

  init() {
    function setEventListener(aId, aEventType, aCallback) {
      document.getElementById(aId)
              .addEventListener(aEventType, aCallback.bind(gAdvancedPane));
    }

    this._inited = true;

    if (AppConstants.MOZ_UPDATER) {
      let onUnload = function() {
        window.removeEventListener("unload", onUnload);
        Services.prefs.removeObserver("app.update.", this);
      }.bind(this);
      window.addEventListener("unload", onUnload);
      Services.prefs.addObserver("app.update.", this);
      this.updateReadPrefs();
      setEventListener("updateRadioGroup", "command",
                       gAdvancedPane.updateWritePrefs);
      setEventListener("showUpdateHistory", "command",
                       gAdvancedPane.showUpdates);
    }
  },

  /*
   * Preferences:
   *
   * app.update.enabled
   * - true if updates to the application are enabled, false otherwise
   * app.update.auto
   * - true if updates should be automatically downloaded and installed and
   * false if the user should be asked what he wants to do when an update is
   * available
   * extensions.update.enabled
   * - true if updates to extensions and themes are enabled, false otherwise
   * browser.search.update
   * - true if updates to search engines are enabled, false otherwise
   */

  /**
   * Selects the item of the radiogroup based on the pref values and locked
   * states.
   *
   * UI state matrix for update preference conditions
   *
   * UI Components:                              Preferences
   * Radiogroup                                  i   = app.update.enabled
   *                                             ii  = app.update.auto
   *
   * Disabled states:
   * Element           pref  value  locked  disabled
   * radiogroup        i     t/f    f       false
   *                   i     t/f    *t*     *true*
   *                   ii    t/f    f       false
   *                   ii    t/f    *t*     *true*
   */
  updateReadPrefs() {
    if (AppConstants.MOZ_UPDATER) {
      var enabledPref = document.getElementById("app.update.enabled");
      var autoPref = document.getElementById("app.update.auto");
      var radiogroup = document.getElementById("updateRadioGroup");

      if (!enabledPref.value)   // Don't care for autoPref.value in this case.
        radiogroup.value = "manual";    // 3. Never check for updates.
      else if (autoPref.value)  // enabledPref.value && autoPref.value
        radiogroup.value = "auto";      // 1. Automatically install updates
      else                      // enabledPref.value && !autoPref.value
        radiogroup.value = "checkOnly"; // 2. Check, but let me choose

      var canCheck = Components.classes["@mozilla.org/updates/update-service;1"].
                       getService(Components.interfaces.nsIApplicationUpdateService).
                       canCheckForUpdates;
      // canCheck is false if the enabledPref is false and locked,
      // or the binary platform or OS version is not known.
      // A locked pref is sufficient to disable the radiogroup.
      radiogroup.disabled = !canCheck || enabledPref.locked || autoPref.locked;

      if (AppConstants.MOZ_MAINTENANCE_SERVICE) {
        // Check to see if the maintenance service is installed.
        // If it is don't show the preference at all.
        var installed;
        try {
          var wrk = Components.classes["@mozilla.org/windows-registry-key;1"]
                    .createInstance(Components.interfaces.nsIWindowsRegKey);
          wrk.open(wrk.ROOT_KEY_LOCAL_MACHINE,
                   "SOFTWARE\\Mozilla\\MaintenanceService",
                   wrk.ACCESS_READ | wrk.WOW64_64);
          installed = wrk.readIntValue("Installed");
          wrk.close();
        } catch (e) {
        }
        if (installed != 1) {
          document.getElementById("useService").hidden = true;
        }
      }
    }
  },

  /**
   * Sets the pref values based on the selected item of the radiogroup.
   */
  updateWritePrefs() {
    if (AppConstants.MOZ_UPDATER) {
      var enabledPref = document.getElementById("app.update.enabled");
      var autoPref = document.getElementById("app.update.auto");
      var radiogroup = document.getElementById("updateRadioGroup");
      switch (radiogroup.value) {
        case "auto":      // 1. Automatically install updates for Desktop only
          enabledPref.value = true;
          autoPref.value = true;
          break;
        case "checkOnly": // 2. Check, but let me choose
          enabledPref.value = true;
          autoPref.value = false;
          break;
        case "manual":    // 3. Never check for updates.
          enabledPref.value = false;
          autoPref.value = false;
      }
    }
  },

  /**
   * Displays the history of installed updates.
   */
  showUpdates() {
    gSubDialog.open("chrome://mozapps/content/update/history.xul");
  },

  observe(aSubject, aTopic, aData) {
    if (AppConstants.MOZ_UPDATER) {
      switch (aTopic) {
        case "nsPref:changed":
          this.updateReadPrefs();
          break;
      }
    }
  },
};
