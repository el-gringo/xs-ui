/**
 * @import { App } from "../types.js";
 *
 *
 * @param {App} app
 */
function initializeElementBindings(app) {
  return {
    element_offset_width(elementPtr) {
      const { heap } = app;
      return heap.get(elementPtr).offsetWidth;
    },
    element_offset_height(elementPtr) {
      const { heap } = app;
      return heap.get(elementPtr).offsetHeight;
    },

    element_set_inner_html(elementPtr, htmlPtr, htmlLen) {
      const { heap, utf8 } = app;
      const html = utf8.fromWasm(htmlPtr, htmlLen);
      heap.get(elementPtr).innerHTML = html;
    },
    element_inner_html(elementPtr) {
      const { heap, bytes } = app;
      return bytes.toCString(heap.get(elementPtr).innerHTML);
    },

    element_set_attribute(
      elementPtr,
      namePtr,
      nameLen,
      valuePtr,
      valueLen,
    ) {
      const { heap, utf8 } = app;
      const name = utf8.fromWasm(namePtr, nameLen);
      const value = utf8.fromWasm(valuePtr, valueLen);
      heap.get(elementPtr).setAttribute(name, value);
    },
    element_attribute(elementPtr, namePtr, nameLen) {
      const { heap, utf8, bytes } = app;
      const name = utf8.fromWasm(namePtr, nameLen);
      const value = heap.get(elementPtr).getAttribute(name);
      if (value === null) {
        return -1;
      }
      return bytes.toCString(value);
    },
    element_remove_attribute(elementPtr, namePtr, nameLen) {
      const { heap, utf8 } = app;
      const name = utf8.fromWasm(namePtr, nameLen);
      heap.get(elementPtr).removeAttribute(name);
    },

    element_set_style(
      elementPtr,
      propertyPtr,
      propertyLen,
      valuePtr,
      valueLen,
    ) {
      const { heap, utf8 } = app;
      const property = utf8.fromWasm(propertyPtr, propertyLen);
      const value = utf8.fromWasm(valuePtr, valueLen);
      heap.get(elementPtr).style.setProperty(property, value);
    },
    element_style(elementPtr, propertyPtr, propertyLen) {
      const { heap, utf8, bytes } = app;
      const property = utf8.fromWasm(propertyPtr, propertyLen);
      return bytes.toCString(
        heap.get(elementPtr).style.getPropertyValue(property),
      );
    },

    element_value(elementPtr) {
      const { heap, bytes } = app;
      return bytes.toCString(heap.get(elementPtr).value ?? "");
    },
    element_set_value(elementPtr, valuePtr, valueLen) {
      const { heap, utf8 } = app;
      heap.get(elementPtr).value = utf8.fromWasm(valuePtr, valueLen);
    },

    element_class_name(elementPtr) {
      const { heap, bytes } = app;
      return bytes.toCString(heap.get(elementPtr).className);
    },
    element_set_class_name(elementPtr, namePtr, nameLen) {
      const { heap, utf8 } = app;
      heap.get(elementPtr).className = utf8.fromWasm(namePtr, nameLen);
    },

    element_class_list(elementPtr) {
      const { heap } = app;
      return heap.add(heap.get(elementPtr).classList);
    },

    class_list_add(listPtr, tokenPtr, tokenLen) {
      const { heap, utf8 } = app;
      heap.get(listPtr).add(utf8.fromWasm(tokenPtr, tokenLen));
    },
    class_list_remove(listPtr, tokenPtr, tokenLen) {
      const { heap, utf8 } = app;
      heap.get(listPtr).remove(utf8.fromWasm(tokenPtr, tokenLen));
    },
    class_list_toggle(listPtr, tokenPtr, tokenLen) {
      const { heap, utf8 } = app;
      return heap.get(listPtr).toggle(utf8.fromWasm(tokenPtr, tokenLen));
    },
    class_list_contains(listPtr, tokenPtr, tokenLen) {
      const { heap, utf8 } = app;
      return heap.get(listPtr).contains(utf8.fromWasm(tokenPtr, tokenLen));
    },

    element_remove(elementPtr) {
      const { heap } = app;
      heap.get(elementPtr).remove();
    },
    element_focus(elementPtr) {
      const { heap } = app;
      heap.get(elementPtr).focus();
    },
  };
}

define(initializeElementBindings);
