{% if render_title %}
## v{{ versiondata.version }} ({{ versiondata.date }})
{% endif %}
{% for section, _ in sections.items() %}
{% if section %}

## {{section}}
{% endif %}

{% if sections[section] %}
{% for category, val in definitions.items() if category in sections[section] %}
### {{ definitions[category]['name'] }}

{% for text, values in sections[section][category].items() %}
- {{ text }}
{%- if values %}
{% if "\n  - " in text or '\n  * ' in text %}


  (
{%- else %}
{% if text %} ({% endif %}
{%- endif -%}
{%- for issue in values %}
{{ issue.split(": ", 1)[0] }}{% if not loop.last %}, {% endif %}
{%- endfor %}
{% if text %}){% endif %}

{% else %}

{% endif %}
{% endfor %}

{% if issues_by_category[section][category] and "]: " in issues_by_category[section][category][0] %}
{% for issue in issues_by_category[section][category] %}
{{ issue }}
{% endfor %}

{% endif %}
{% if sections[section][category]|length == 0 %}
No significant changes.

{% else %}
{% endif %}
{% endfor %}
{% else %}
No significant changes.

{% endif %}
{% endfor +%}
