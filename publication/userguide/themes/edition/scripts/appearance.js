(function () {
  "use strict";

  var appearances = ["parchment", "paper", "night"];
  var root = document.documentElement;
  var storageKey = "spore.edition.appearance.v1";
  var appearance = "night";

  try {
    var stored = window.localStorage.getItem(storageKey);
    if (appearances.indexOf(stored) !== -1) {
      appearance = stored;
    }
  } catch (_error) {
    // Storage is optional; this publication defaults to Night.
  }

  root.setAttribute("data-edition-appearance", appearance);
  root.setAttribute("data-edition-appearance-ready", "true");

  function initializePicker() {
    var picker = document.querySelector(".spore-appearance-picker");
    if (!picker) {
      return;
    }

    var choices = Array.prototype.slice.call(
      picker.querySelectorAll("[data-edition-appearance-choice]")
    );
    var pointerWasOpen = false;

    function currentAppearance() {
      var current = root.getAttribute("data-edition-appearance");
      return appearances.indexOf(current) === -1 ? "night" : current;
    }

    function isOpen() {
      return picker.getAttribute("data-edition-appearance-open") === "true";
    }

    function arrangeChoices() {
      var current = currentAppearance();
      var ordered = choices.filter(function (choice) {
        return choice.getAttribute("data-edition-appearance-choice") === current;
      }).concat(choices.filter(function (choice) {
        return choice.getAttribute("data-edition-appearance-choice") !== current;
      }));

      ordered.forEach(function (choice, slot) {
        var selected = slot === 0;
        choice.setAttribute("data-edition-appearance-slot", String(slot));
        choice.setAttribute("aria-pressed", selected ? "true" : "false");
        choice.removeAttribute("aria-expanded");
      });
    }

    function setOpen(open) {
      var expanded = open ? "true" : "false";
      picker.setAttribute("data-edition-appearance-open", expanded);

      choices.forEach(function (choice) {
        var anchor = choice.getAttribute("data-edition-appearance-slot") === "0";
        choice.tabIndex = open || anchor ? 0 : -1;
        if (anchor) {
          choice.setAttribute("aria-expanded", expanded);
        } else {
          choice.removeAttribute("aria-expanded");
        }
      });
    }

    function selectAppearance(name) {
      if (appearances.indexOf(name) === -1) {
        return;
      }

      root.setAttribute("data-edition-appearance", name);
      try {
        window.localStorage.setItem(storageKey, name);
      } catch (_error) {
        // Selection remains active for this page when storage is unavailable.
      }

      arrangeChoices();
      setOpen(isOpen());
    }

    arrangeChoices();
    setOpen(false);

    picker.addEventListener("pointerenter", function (event) {
      if (event.pointerType !== "touch") {
        setOpen(true);
      }
    });

    picker.addEventListener("pointerleave", function () {
      if (!picker.contains(document.activeElement)) {
        setOpen(false);
      }
    });

    picker.addEventListener("focusin", function () {
      setOpen(true);
    });

    picker.addEventListener("focusout", function () {
      window.setTimeout(function () {
        if (!picker.contains(document.activeElement)) {
          setOpen(false);
        }
      }, 0);
    });

    picker.addEventListener("pointerdown", function () {
      pointerWasOpen = isOpen();
    });

    picker.addEventListener("click", function (event) {
      var choice = event.target.closest("[data-edition-appearance-choice]");
      if (!choice || !picker.contains(choice)) {
        return;
      }

      var name = choice.getAttribute("data-edition-appearance-choice");
      if (name === currentAppearance()) {
        setOpen(event.detail === 0 ? !isOpen() : !pointerWasOpen);
        return;
      }

      selectAppearance(name);
      setOpen(false);
    });

    picker.addEventListener("keydown", function (event) {
      if (event.key !== "Escape") {
        return;
      }

      event.preventDefault();
      setOpen(false);
      var anchor = picker.querySelector('[data-edition-appearance-slot="0"]');
      if (anchor) {
        anchor.focus();
      }
    });

    document.addEventListener("pointerdown", function (event) {
      if (!picker.contains(event.target)) {
        setOpen(false);
      }
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initializePicker);
  } else {
    initializePicker();
  }
}());
