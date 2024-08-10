class GameBoyAdvanceKeypad {
	constructor() {
		this.KEYCODE_LEFT = 37; //Arrow keys
		this.KEYCODE_UP = 38;
		this.KEYCODE_RIGHT = 39;
		this.KEYCODE_DOWN = 40;
		this.KEYCODE_START = 13; //Enter
		this.KEYCODE_SELECT = 8; //Backslash
		this.KEYCODE_A = 83; //s
		this.KEYCODE_B = 65; //a
		this.KEYCODE_L = 81; //q
		this.KEYCODE_R = 87; //r

		this.GAMEPAD_LEFT = 14;
		this.GAMEPAD_UP = 12;
		this.GAMEPAD_RIGHT = 15;
		this.GAMEPAD_DOWN = 13;
		this.GAMEPAD_START = 9;
		this.GAMEPAD_SELECT = 8;
		this.GAMEPAD_A = 1;
		this.GAMEPAD_B = 0;
		this.GAMEPAD_L = 4;
		this.GAMEPAD_R = 5;
		this.GAMEPAD_THRESHOLD = 0.2;

		this.A = 0;
		this.B = 1;
		this.SELECT = 2;
		this.START = 3;
		this.RIGHT = 4;
		this.LEFT = 5;
		this.UP = 6;
		this.DOWN = 7;
		this.R = 8;
		this.L = 9;

		this.currentDownKeyboard = 0x0;
		this.currentDownGamepad = [0x0, 0x0, 0x0]; //up to 3 gamepads supported simultanously
		this.eatInput = false;

		this.gamepads = [];

		this.remappingKeyId = "";
	}
  currentDownState() {
    return this.currentDownKeyboard | this.currentDownGamepad[0] | this.currentDownGamepad[1] | this.currentDownGamepad[2];
  }
	keyboardHandler(e) {
		var toggle = 0;

		// Check for a remapping
		if (this.remappingKeyId != "") {
			this.remapKeycode(this.remappingKeyId, e.keyCode);
			this.remappingKeyId = "";
			e.preventDefault();
			return; // Could do an else and wrap the rest of the function in it, but this is cleaner
		}

		switch (e.keyCode) {
			case this.KEYCODE_START:
				toggle = this.START;
				break;
			case this.KEYCODE_SELECT:
				toggle = this.SELECT;
				break;
			case this.KEYCODE_A:
				toggle = this.A;
				break;
			case this.KEYCODE_B:
				toggle = this.B;
				break;
			case this.KEYCODE_L:
				toggle = this.L;
				break;
			case this.KEYCODE_R:
				toggle = this.R;
				break;
			case this.KEYCODE_UP:
				toggle = this.UP;
				break;
			case this.KEYCODE_RIGHT:
				toggle = this.RIGHT;
				break;
			case this.KEYCODE_DOWN:
				toggle = this.DOWN;
				break;
			case this.KEYCODE_LEFT:
				toggle = this.LEFT;
				break;
			default:
				return;
		}

		toggle = 1 << toggle;
		if (e.type == "keydown") {
			this.currentDownKeyboard |= toggle;
		} else {
			this.currentDownKeyboard &= ~toggle;
		}

		if (this.eatInput) {
			e.preventDefault();
		}
	}
	gamepadHandler(gamepad, idx) {
		var value = 0;
    if(gamepad) {
      if (this.GAMEPAD_LEFT<gamepad.buttons.length && gamepad.buttons[this.GAMEPAD_LEFT].value > this.GAMEPAD_THRESHOLD) {
        value |= 1 << this.LEFT;
      }
      if (this.GAMEPAD_UP<gamepad.buttons.length && gamepad.buttons[this.GAMEPAD_UP].value > this.GAMEPAD_THRESHOLD) {
        value |= 1 << this.UP;
      }
      if (this.GAMEPAD_RIGHT<gamepad.buttons.length && gamepad.buttons[this.GAMEPAD_RIGHT].value > this.GAMEPAD_THRESHOLD) {
        value |= 1 << this.RIGHT;
      }
      if (this.GAMEPAD_DOWN<gamepad.buttons.length && gamepad.buttons[this.GAMEPAD_DOWN].value > this.GAMEPAD_THRESHOLD) {
        value |= 1 << this.DOWN;
      }
      if (this.GAMEPAD_START<gamepad.buttons.length && gamepad.buttons[this.GAMEPAD_START].value > this.GAMEPAD_THRESHOLD) {
        value |= 1 << this.START;
      }
      if (this.GAMEPAD_SELECT<gamepad.buttons.length && gamepad.buttons[this.GAMEPAD_SELECT].value > this.GAMEPAD_THRESHOLD) {
        value |= 1 << this.SELECT;
      }
      if (this.GAMEPAD_A<gamepad.buttons.length && gamepad.buttons[this.GAMEPAD_A].value > this.GAMEPAD_THRESHOLD) {
        value |= 1 << this.A;
      }
      if (this.GAMEPAD_B<gamepad.buttons.length && gamepad.buttons[this.GAMEPAD_B].value > this.GAMEPAD_THRESHOLD) {
        value |= 1 << this.B;
      }
      if (this.GAMEPAD_L<gamepad.buttons.length && gamepad.buttons[this.GAMEPAD_L].value > this.GAMEPAD_THRESHOLD) {
        value |= 1 << this.L;
      }
      if (this.GAMEPAD_R<gamepad.buttons.length && gamepad.buttons[this.GAMEPAD_R].value > this.GAMEPAD_THRESHOLD) {
        value |= 1 << this.R;
      }
      
    }

		this.currentDownGamepad[idx] = value & 0x3ff;
	}
	gamepadConnectHandler(gamepad) {
		this.gamepads.push(gamepad);
	}
	gamepadDisconnectHandler(gamepad) {
		this.gamepads = self.gamepads.filter(function (other) {
			return other != gamepad;
		});
	}
	pollGamepads() {
		var navigatorList = [];
		if (navigator.webkitGetGamepads) {
			navigatorList = navigator.webkitGetGamepads();
		} else if (navigator.getGamepads) {
			navigatorList = navigator.getGamepads();
		}

		// Let's all give a shout out to Chrome for making us get the gamepads EVERY FRAME
		/* How big of a performance draw is this? Would it be worth letting users know? */
		if (navigatorList.length) {
			this.gamepads = [];
		}
		for (var i = 0; i < navigatorList.length; ++i) {
			if (navigatorList[i]) {
				this.gamepads.push(navigatorList[i]);
			}
		}
		if (this.gamepads.length > 0) {
      for (var i = 0; i < 3; ++i) {
        this.gamepadHandler(i<this.gamepads.length ? this.gamepads[i] : null, i);
      }
		}
	}
	registerHandlers() {
		window.addEventListener(
			"keydown",
			this.keyboardHandler.bind(this),
			true
		);
		window.addEventListener("keyup", this.keyboardHandler.bind(this), true);

		window.addEventListener(
			"gamepadconnected",
			this.gamepadConnectHandler.bind(this),
			true
		);
		window.addEventListener(
			"mozgamepadconnected",
			this.gamepadConnectHandler.bind(this),
			true
		);
		window.addEventListener(
			"webkitgamepadconnected",
			this.gamepadConnectHandler.bind(this),
			true
		);

		window.addEventListener(
			"gamepaddisconnected",
			this.gamepadDisconnectHandler.bind(this),
			true
		);
		window.addEventListener(
			"mozgamepaddisconnected",
			this.gamepadDisconnectHandler.bind(this),
			true
		);
		window.addEventListener(
			"webkitgamepaddisconnected",
			this.gamepadDisconnectHandler.bind(this),
			true
		);
	}
	// keyId is ["A", "B", "SELECT", "START", "RIGHT", "LEFT", "UP", "DOWN", "R", "L"]
	initKeycodeRemap(keyId) {
		// Ensure valid keyId
		var validKeyIds = ["A", "B", "SELECT", "START", "RIGHT", "LEFT", "UP", "DOWN", "R", "L"];
		if (validKeyIds.indexOf(keyId) != -1) {
			/* If remappingKeyId holds a value, the keydown event above will
			wait for the next keypress to assign the keycode */
			this.remappingKeyId = keyId;
		}
	}
	// keyId is ["A", "B", "SELECT", "START", "RIGHT", "LEFT", "UP", "DOWN", "R", "L"]
	remapKeycode(keyId, keycode) {
		switch (keyId) {
			case "A":
				this.KEYCODE_A = keycode;
				break;
			case "B":
				this.KEYCODE_B = keycode;
				break;
			case "SELECT":
				this.KEYCODE_SELECT = keycode;
				break;
			case "START":
				this.KEYCODE_START = keycode;
				break;
			case "RIGHT":
				this.KEYCODE_RIGHT = keycode;
				break;
			case "LEFT":
				this.KEYCODE_LEFT = keycode;
				break;
			case "UP":
				this.KEYCODE_UP = keycode;
				break;
			case "DOWN":
				this.KEYCODE_DOWN = keycode;
				break;
			case "R":
				this.KEYCODE_R = keycode;
				break;
			case "L":
				this.KEYCODE_L = keycode;
				break;
		}
	}
}
